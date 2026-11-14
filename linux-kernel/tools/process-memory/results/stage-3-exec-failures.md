# Rejected child exec and descriptor lifetime

The [closed composable archive](stage-3-exec-failures/report.json) and
[frozen replay review](stage-3-exec-failures-review.json) extend the preceding
[signal cohort](stage-3-exec-signals.md) with sixteen ordinary rejected child
`execve` calls followed by four successful replacements.

This tests existing `TOPAL-SYSTEMS-PROCESS-MM-001`,
`TOPAL-SYSTEMS-STATIC-PROCESS-EXEC-ARGUMENTS-001`,
`TOPAL-SYSTEMS-STATIC-PROCESS-EXEC-ENVIRONMENT-001`,
`TOPAL-SYSTEMS-STATIC-PROCESS-SIGNAL-ACTION-001`,
`TOPAL-SYSTEMS-STATIC-PROCESS-SIGNAL-MASK-001`,
`TOPAL-SYSTEMS-STATIC-PROCESS-FD-FLAGS-001` and
`TOPAL-SYSTEMS-STATIC-PROCESS-FD-CLOSE-001` obligations. It changes validation,
not the approved design, kernel instructions or production selection.

The parent opens a descriptor before fork. The child independently enables its
close-on-exec flag, creates nonempty signal/TLS/registration state and grows its
private heap to 101 pages, retaining a separate 67-page parent heap. It attempts
an absent executable (`ENOENT`), an inaccessible pathname (`EFAULT`), an
inaccessible argv vector (`EFAULT`) and an inaccessible environment vector after
valid argv (`EFAULT`). Invalid pointers are normal syscall arguments; no debugger
changes guest registers, allocations, mappings or kernel state.

After every rejection, the application queries signal actions, blocked mask,
break, TLS and descriptor flags and checks its dirty heap bytes. The read-only
observer independently records the ordinary call and actual userspace return:
process/MM identities, active/inactive states and ownership tokens, full active
and inactive heap hashes, CR3, TLS, TID/robust registrations, selected signal
actions/mask, sixteen descriptor slots and both sixteen-cell description banks
remain equal. Stack/return address and System V callee-preserved registers match
the ordinary continuation. Successful exec then closes the child's descriptor,
resets actions and preserves the blocked mask. Its new C entry queries the
descriptor number passed through ordinary argv and gets `EBADF`. The parent
still sees its original descriptor without the child's close-on-exec flag,
closes it and restores its original signal state.

The targeted native test passes all four generations and the surrounding ABI
witness. Independent readback passes all sixteen error returns, four later
successful execs, eight new altered-record controls, nine preceding signal
controls and the existing process/exec controls. The new controls reject changed
child/parent heap hashes, lost TLS/TID/robust state, changed MM identities, lost
descriptors and changed description state. They alter saved Python records only.
All **82 process-memory** and **248 user-memory** models pass. Conservative C
stack use is **1,696 / 2,048 bytes**. No native attempt or reader failed in this
stage; the predecessor archive retains its earlier reader failure separately.

The archive has **145 entries** and explicitly depends on the preceding
1,021-entry signal archive, whose manifest is pinned to
`335595b6c687e48f6606ba0f8da73d9bbbe284d38a0dcd9b294fd1b5167ea5e3`.
It reuses 744 exact source inputs and adds eight overlays, restoring 752 inputs
for frozen readback/control checks and all 330 models. Its own manifest is
`257079b0d45900342497fe7a3dd6d08ad82b2e30d90a9b67517a942731fd4bfb`.
Frozen replay executes no VM, compiler or application binary.

The kernel remains
`b8212e5e9aca715d8db0d863cbc9655ed989c3884b938b57055c3e22e04d0a7c`,
and the boot image remains
`6efb7d19f9bab3b2576cf6dfc173e734f6de9a0f7db9d3cfdf1950392b2371b2`.
Only the first-party source-built resource changes; pinned third-party binaries
are not patched. Native/model peak/limit is
**928,292,864 / 3,395,125,248 bytes**; collection/replay is
**1,074,167,808 / 3,405,209,600 bytes**. Both scopes record zero swap and zero
sampled memory events under the normal resource wrapper.

Risk is moderate validation-coverage risk. Single-agent review checks approved
contracts, pinned Linux 7.2.9 `fs/exec.c` and descriptor/signal behavior, generated
stack bounds, independent saved-state comparisons, altered-record controls,
kernel/image identities and frozen replay. This covers selected returning
precommit errors. Linux's post-point-of-no-return failures, other exec errors,
all syscall register obligations, unobserved paging/CPU state, continuous
ownership/exclusion, signal delivery, public production reconstruction, the
interrupted full compiler suite and full distribution/container/VM-host goals
remain unfinished. The 51-application corpus was not rerun on this resource.
No outcome explains an earlier AI safeguard intervention.

Run a fresh bounded native cohort with:

```sh
rtk scripts/run_bounded.py -- python3 linux-kernel/tools/process-memory/run_exec_signals.py linux-kernel/.cache/x86_64/exec-failures-new --failed-exec
```

`readback_exec_failures.py` checks its saved observations.
`archive_exec_failures.py` collects a completed run over the pinned predecessor
archive and restores exact frozen inputs for record/model replay.
