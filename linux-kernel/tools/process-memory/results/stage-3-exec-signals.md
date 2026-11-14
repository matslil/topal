# Ordinary child-exec signal state

The [closed archive](stage-3-exec-signals/report.json) and
[frozen replay review](stage-3-exec-signals-review.json) record four fresh
source-built child-exec scenarios inside the authorized x86-64 Q35 VM.
This extends `TOPAL-SYSTEMS-PROCESS-MM-001` with existing
`TOPAL-SYSTEMS-STATIC-PROCESS-SIGNAL-ACTION-001` and
`TOPAL-SYSTEMS-STATIC-PROCESS-SIGNAL-MASK-001` obligations. It changes validation,
not language design, kernel instructions, production selection or security rules.

The application installs ignored `SIGQUIT` and caught `SIGUSR1` dispositions
with nonempty flags, restorer and action masks. `fork` must copy both actions
and the blocked mask. The child independently changes its action metadata and
blocked set, then calls ordinary `execve`. Its new image queries the actions:
the ignored handler survives, the caught handler becomes default, every action's
metadata clears, and the child's blocked set survives. These checks repeat
after a normal yield. The parent checks its own unchanged state, waits for
the expected child status and restores its original registrations and mask.
`SIGCHLD` is left unchanged. No signal is sent or delivered.

The observer only reads guest state. Four ordinary userspace `execve` call
observations bind nonempty input state to the corresponding process/MM identity,
four memory-gateway transitions and four child image entries. Four complete
case returns prove that the parent restored its original signal state. The C
program's success also retains the existing heap, TLS, registration, yield,
wait-output-retry and full surrounding ABI-witness checks. Its conservative
generated stack bound is **1,616 / 2,048 bytes**.

The first native execution passed all four cases, but the initial independent
reader failed: it expected unreset actions at the memory gateway, which runs
after signal reset. The corrected observer captures the ordinary application
call before syscall entry; a second complete native execution and reader pass.
The archive retains the first observations, logs and original reader. Frozen
replay reproduces its exact `ignored and caught input dispositions` refusal.
An initial archive collection also failed on integer versus JSON string keys
in operation counts; canonicalizing JSON fixed that comparison. This collection
error caused no native execution or kernel change.

The corrected run passes **82 process-memory models**, **248 user-memory models**
and **nine saved-record signal controls**, alongside the existing process/exec
controls. Those nine controls alter Python records only: lost ignored handler,
retained caught handler, retained flags/restorer/action mask, lost blocked mask,
changed image-entry mask, vacuous input mask and changed parent mask must fail.
The 1,021-entry archive freezes 750 inputs; frozen replay repeats the corrected
reader, the original refusal and all 330 models without building or running a VM.

The kernel remains byte-identical to the previous stage:
`b8212e5e9aca715d8db0d863cbc9655ed989c3884b938b57055c3e22e04d0a7c`.
The boot image remains
`6efb7d19f9bab3b2576cf6dfc173e734f6de9a0f7db9d3cfdf1950392b2371b2`.
Only the first-party source-built application resource changes. Pinned external
application binaries are not patched. The archive manifest SHA-256 is
`335595b6c687e48f6606ba0f8da73d9bbbe284d38a0dcd9b294fd1b5167ea5e3`.

The corrected native/model batch records peak/limit
**727,113,728 / 3,389,124,608 bytes**. Collection/frozen replay records
**934,866,944 / 3,404,443,648 bytes**. Both scopes record zero swap and zero
sampled memory events. All heavy commands use the normal resource wrapper;
host security and VM boundaries remain in place.

Risk is moderate: an incorrect observer or reader could overstate lifecycle
coverage. Review is single-agent, using the approved design and formal rules,
the pinned Linux `flush_signal_handlers` implementation, independent saved-state
comparisons, changed-record controls, exact kernel/image identities and frozen
replay. These checks qualify the selected registration/query scenarios only.
They do not establish signal delivery, pending queues, signal frames, restart,
`rt_sigreturn`, every helper path, continuous ownership/exclusion, the interrupted
full compiler suite, production reconstruction, the 51-application corpus on
this new resource, or complete distribution/container/VM-host compatibility.
No result here explains any earlier AI safeguard intervention.

Reproduce the native cohort in a new output directory:

```sh
rtk scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/run_exec_signals.py linux-kernel/.cache/x86_64/exec-signals-new
```

This requires the repository's pinned QEMU/GDB/LLVM tools and existing test
inputs. `readback_exec_signals.py` separately checks saved observations.
`archive_exec_signals.py` collects a completed run and its earlier reader attempt,
checks archive closure, restores exact frozen inputs and repeats record/model
checks. Its replay does not constitute another native execution.
