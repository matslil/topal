# Stage 3: selected owner records guarded through copying

`TK-DEC-283`, `TOPAL-SYSTEMS-USER-EXTENT-001`.

The [read-only interval observer](../../../../labs/qemu/x86_64/user_extent_interval_observer.py)
extends the preceding helper entry/return observations through the actual final
copy instruction. All **52 successful stat/positional-write transfers** retain
**26 protected ranges** each: six complete translation pages, root/heap tokens,
allocator/MM/entry fields, five live registry records and all corresponding
allocation cells. The range and comparison [model](../../owner_interval.py)
checks exact geometry and permits only monotonic hardware Accessed/Dirty sets
on the admitted paths. Permissions, addresses, provenance, pins, identities and
unrelated status remain fixed.

These are GDB hardware write watchpoints at identity virtual addresses. They
detect value-changing CPU stores, including a change followed by restoration;
same-value stores need not report. Their absence does not establish exclusion
through a different virtual alias, DMA, control-register changes/restoration,
or all asynchronous entries. This remains a selected observation rather than
a general mapping/ownership loan or complete lifetime proof.

Every iteration requires ordinary `brk` edits to trigger watches on a complete
translation page, MM state and a live heap registry record: **twelve actual
positive controls** in total. Separate unchanged-kernel pilots establish a
4096-byte range hit and a hit in the final eight bytes of a **12072-byte** range,
the largest selected allocation-cell span. The latter changes byte offsets
12065 and 12066, using a page-table edit to test that range size. They stop at
the ordinary edit and do not complete the whole
application. Neither pilot assigns guest registers, mappings or private state.

The [complete native run](native/run.json) passes all four metadata iterations
and enclosing process retirement in about **549 seconds**.
It retains **104 boundary observations**, stored as
**64 distinct interval snapshots**, and
**27 created hardware watchpoints**, with zero detected
protected identity-address CPU-store changes. All selected copies complete at
their exact verified stat/pwrite instruction endpoints. Owner/helper readback
still joins all 44 stat outputs and eight positional-write inputs, and metadata
readback replays all 136 syscalls, 124 complete filesystem/table transitions,
44 stat outputs and 44 actual clock returns.

Every range remains armed through its selected copy. After all 52 copies and
twelve controls, all inactive watchpoints are deleted before the independent
enclosing application retirement. The retained intermediate progress file
explicitly says that full guest retirement is not yet complete; only the later
complete run and observations establish that result. The deadline remains
600 seconds, and guest workloads, host boundaries and bounded resource limits
are retained. Native peak memory is **594006016 bytes**,
with zero swap and zero memory-event counters. The VM retains its actual two-CPU
TCG configuration; no single-CPU assumption is made.

All **155 model/reader tests** and **fourteen offline saved-record controls**
pass. Controls reject incomplete ranges, false watch mode/disarm/count/change
claims, wrong copy endpoints, unrun controls, missing intervals and coherently
rebound permission/address/status/entry changes. They alter JSON records only;
no changed ELF, debugger state, register or guest memory is executed.

[Artifact comparison](artifact-comparison.json) preserves seventeen core
objects/images, thirteen audits and four other metadata artifacts byte-for-byte.
Kernel SHA-256 remains
`47b785aca3794d171d508a3fdbb616057a613016c1783ca9ec95236dbba2199f`.
First-party source/fixture/boot-resource bytes and all five changed runtime/test
input versions remain bound. Filename/provenance differences retain their actual
versions without a retained intermediate-text equivalence claim. No new GNU
decoder is run. The previous reach counts and conditional index keep their
limited scope; executable kernel behavior, core language meaning and canonical
production selection are unchanged.
The complete [7606-byte console transcript](console-readback.json) also matches
the preceding same-kernel cohort exactly.

The [initial full run](history/timeout/status.json) failed its 600-second debugger
deadline after four source iterations, with zero memory-event counters. Its
five exact input versions, raw debugger/serial records and failed status remain.
The subsequent change deletes inactive watchpoints after every selected interval
has completed, without reducing their number, controls, coverage or deadline.
Later success does not itself establish the original timeout's cause. The early
pilot's invalid negative-breakpoint-number info query is also retained separately;
it is a debugger query error, not an unsafe-operation or provider denial.

The [source record](source-identity.json) binds **178 selected inputs** and the
[artifact record](artifact-identity.json) binds **93 current/historical artifacts**.
[Frozen replay](replay.json) restores selected sources/artifacts, reproduces all
thirteen reports, all 155 models, exact interval/owner/metadata readbacks and the
fourteen rejected controls. It executes no ELF, compiler, GNU decoder, GDB or VM:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-owner-intervals/replay.py linux-kernel/.cache/x86_64/owner-interval-replay-new
```

Use a new output directory. [Validation](validate-archive.py) binds retained
sources, snapshots, native records and supporting evidence. [Tool facts](tools.json)
distinguish installed QEMU 10.2.2/GDB 17.2 from consulted current documentation.
QEMU documents TCG system-emulation watchpoint support; GDB documents changed-value
watch semantics and separate remote limits. Sources: [QEMU](https://www.qemu.org/docs/master/system/gdb.html#breakpoint-and-watchpoint-support),
[GDB watchpoints](https://sourceware.org/gdb/current/onlinedocs/gdb.html/Set-Watchpoints.html),
[GDB remote configuration](https://www.sourceware.org/gdb/current/onlinedocs/gdb.html/Remote-Configuration.html).
Actual pilots supply installed-version evidence.

Review remains single-agent with separate source/ELF/model/saved-record
interpretations and high memory/lifetime/evidence risk; no independent human
security review is claimed. Physical-alias/DMA/control-register/asynchronous
exclusion, continuous stable lifetime, other callers/private-counterpart loans,
complete exception/fatal/whole-syscall effects, unrun native lengths/refusals and
all fallback gates remain required. Canonical clean production reconstruction
and full Linux distribution/container/VM-host acceptance remain unfinished.
Stage 3, initial Step 4 and the original kernel objective remain open.
