# Stage 3: RELRO and exec image publication

`TK-DEC-283`, `TOPAL-SYSTEMS-USER-EXTENT-001`,
`TOPAL-COMP-SYSTEMS-USER-EXTENT-001`.

The [read-only observer](../../../../labs/qemu/x86_64/user_extent_image_observer.py)
records **four RELRO publications** and **three exec image restorations** in a
fresh complete ordinary metadata VM. The kernel, provider, source application
and boot-resource bytes remain unchanged. This increment changes validation
only and does not activate the replacement user-memory validator.

The [layout inspector](../../inspect_image_publication.py) binds the complete
current **100-byte** RELRO store/check/invalidation/barrier loop and **638-byte**
exec unrolled image-leaf reset/invalidation loop. All selected sites belong to
the preceding linked process graph, and these are its only two `INVLPG` sites.
That graph retains its own root/callee/alternate-entry limitations. The inspector
is an exact-layout reader, not a general x86 decoder or full source producer.
The preceding LLVM/GNU inventory remains separate evidence; it is not rerun.

At **21 actual owner boundaries**, the [model](../../image_publication.py) checks
the selected allocator, root tokens, live records, allocation cells, pin counts,
disjoint backing, current context, entry epoch and paging mode. The changed
image leaves retain their actual physical image owner. RELRO removes write
permission from eight NX user leaves; exec restores eighteen writable NX user
leaves. Every word of all six recorded tables is compared. Other descriptors
remain unchanged apart from permitted monotonic hardware A/D sets.

First-image exec is bound to the actual inactive-generation flag and phase 32.
Later exec is bound to the active process lifetime and the existing admitted
startup phase range, 11 through 15. This run observes phase **15** in both later
execs. Every RELRO publication observes phase **8**. These are the existing
[generation checks](../../../../../src/topal-compiler/src/systems_process_runtime.rs),
not new kernel admissions. Models cover the other admitted phase values; native
execution of those values is not claimed here. Temporary image-write restoration
does not grant ordinary user-buffer admission.

QEMU per-CPU logs retain all **86 post-INVLPG states**: eight per RELRO operation
and eighteen per exec restoration. Actual addresses, countdowns, CPU and control
state match their records. Every image-site header in the complete raw logs
belongs to exactly one publication interval. Missing whole generations, reused
trace slices, unmarked duplicates and malformed or incomplete intervals fail.
No image block was explicitly cancelled in this run; the shared parser retains
its separately tested cancellation/retry handling.

Publication precedes invalidation in both layouts. RELRO's final MFENCE precedes
its checked completion. Exec's observed loop ends before the first restored-image
copy; no extra final MFENCE is present or claimed. The distinct existing layouts
are retained rather than inferred from the heap publisher's two-barrier shape.

The complete VM and enclosing retirement pass in **362.560 seconds** under the
unchanged **600-second deadline**. Peak cgroup memory is **395517952 bytes**,
with zero swap and zero memory-event counters. Twelve selected consumed source
identities and fourteen artifact identities remain equal before/after the run.
The actual two-CPU TCG VM, loopback debugger, filesystem/network boundaries and
host protections remain. All **7606 console bytes** match the preceding
same-kernel cohort exactly.

Fresh readback also preserves the preceding **20 heap publications**, **812 heap
post-INVLPG states**, final flushing CR3 handoff and **16 old-backing releases**
with **2228224 bytes** zero before release. Owner readback still checks **52
helpers/104 snapshots**. Metadata readback retains **136 syscalls, 124 complete
transitions, 44 stat outputs and 44 clock returns**. No protected identity-address
watch intervals are claimed newly rerun in this cohort.

All **196 model/reader tests**, [sixteen image controls](checks/image-controls.json)
and eighteen heap controls pass. Controls change saved JSON only. A regression
test replays the actual observer protocol, including repeated RELRO loop-head
visits and rearming after completion, without a guest. This complements model
checks; it does not replace native evidence.

Two complete failed attempts retain exact selected source snapshots, commands,
raw outputs and incomplete status:

- `loop-head-entry` stopped after **83.565 seconds** when the observer treated
  the second RELRO loop-head visit as a new publication. The entry probe is now
  disabled through that complete publication and rearmed only at completion.
- `later-exec-generation` stopped after **186.231 seconds** and two source
  iterations. The model had applied the first-generation phase to later exec.
  Its failed entry's local capture was not saved. The refined observer records
  the actual generation flag and retains active capture before checking it.
  The existing kernel generation guard, protections and workload are unchanged.

Both failures have unchanged source/artifact identities during their runs and
zero swap/resource-event counters. A separate
[wrapper-lock refusal](history/wrapper-lock.json) was a filesystem-sandbox error;
normal approved escalation allowed the same bounded tests to use their existing
host-budget lock. It is not attributed to a model/provider safeguard. No such
safeguard notice occurred during this increment.

The [input delta](inputs.json) adds ten source snapshots over preceding packed
inputs, yielding **201 selected sources and 94 inherited artifacts**. No new
ELF, initramfs or third-party binary is added to Git. [Frozen replay](replay.json)
restores the merged selected inputs, reruns 196 models and reproduces exact
layout/image/heap/owner/metadata and both control reports using Python only:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-image-publication/replay.py linux-kernel/.cache/x86_64/image-publication-replay-new
```

Use a new output directory. [Validation](validate-archive.py) binds current
selected sources, packed/raw identities, native records, failure scopes, logs,
replay and links. Hashes bind bytes; they cannot authenticate observations or
create mapping authority.

Risk is high for permission, owner/lifetime and evidence errors. Review is
single-agent higher-authority/source/diff review plus exact linked-byte models,
ordinary native observations, saved-data controls and frozen replay. Observer
and readback share the model/parser; no independent human security review is
claimed. Core-language meaning and executed kernel/protection behavior remain
unchanged.

Other mapping editors and entry/callee effects, runtime-generated/boot/AP code,
continuous paging mode/TLB history, AP/DMA/asynchronous exclusion and general
stable lifetime remain unqualified. Remaining callers/private-counterpart loans,
complete syscall/fatal/exception effects, unrun native paths, all fallback gates
and canonical clean production reconstruction/activation remain required.
Stage 3, initial Step 4 and full Linux distribution, container and VM-host
acceptance remain unfinished.
