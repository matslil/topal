# Stage 3: local heap publication, invalidation and detached cleanup

`TK-DEC-282`, `TK-DEC-283`, `TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001`,
`TOPAL-SYSTEMS-USER-EXTENT-001`.

The [read-only observer](../../../../labs/qemu/x86_64/user_extent_translation_observer.py)
records **20 heap publications across 36 actual memory requests** in the ordinary
four-iteration metadata VM. Nineteen publications edit the active user root.
The final publication revokes the whole heap after the completed process has
returned to its saved bootstrap root. Kernel, provider and application bytes
are unchanged; this increment adds validation instrumentation only.

The [instruction inspector](../../inspect_translation_publication.py) binds the
complete **149-byte** publisher, its linked PTE helper, current source and exact
compiled `dispatch.ll` block. Separate GNU disassembly supplies another native
interpretation. It also binds the unique saved-root load and CR3-write sequence
in the actual linked process-entry function. Unknown code layouts are rejected;
this template is not a general x86 decoder or another architecture's proof.

At every publication, the [model](../../translation_publication.py) checks every
word of all six recorded tables. New/revoked heap leaves must cover the larger
of the old and new page counts, including removed tails and the low/high table
split. Unrelated descriptors remain unchanged apart from permitted monotonic
hardware status sets. MM state, tokens and context remain fixed during the
publisher. The first publication barrier precedes invalidation; its complete
native loop and final barrier precede the verified return.

QEMU's per-CPU execution logs retain **812 post-INVLPG loop observations**:
**778 in the active user context** and **34 in the bootstrap context after
detachment**. The [trace reader](../../qemu_publication_trace.py) binds actual
addresses, indices, CPU, CR0/CR3/CR4, EFER and critical flags. Every expected page
has its own recorded state. Two explicitly cancelled execution-block attempts
remain separate and require identical matching retries; unmarked duplicates,
missing pages, different retries and terminal cancellations are rejected.

The [saved readback](checks/publication.json) joins those raw log slices to every
publication and request. QEMU reports state before entering an execution block;
INVLPG ends the preceding block in the consulted version. A stopped-before-start
message therefore does not describe a second invalidation. The raw logs and
cancelled states are retained, not silently deduplicated. Default block chaining
is preserved. The allocator-release breakpoint is enabled throughout each memory
request and disabled between requests, where its callback previously returned
without checking anything. All selected release observations remain required.

The final cleanup has a distinct checked context. Native before/after CR3 sites
observe the user-to-bootstrap handoff with PCID and global paging disabled. The
model requires that exact completed handoff, the terminal process state and
whole-heap revocation before accepting detached cleanup. Its subsequent INVLPGs
operate on the current bootstrap context; they do not invalidate an inactive
user root. The preceding observed CR3 replacement supplies the local flush.
This case cannot authorize growth or ordinary user-buffer transfers. The
existing memory gateway already distinguishes these contexts; no admission rule
or protection check is removed or changed.

All **16 old-backing releases** follow publication and invalidation, with actual
unpinned registry records and **2228224 bytes** observed completely zero before
release. The complete VM and enclosing retirement pass in **339.145 seconds**
under the unchanged **600-second deadline**. Peak cgroup memory is
**388837376 bytes**, with zero swap and zero memory-event counters. The actual
two-CPU TCG VM, loopback debugger, filesystem/network boundaries and host
protections remain. The **7606-byte complete console** equals the preceding
same-kernel cohort exactly. Twelve selected consumed artifact identities and
seven modified runtime/test source identities remain equal before/after the run.

Fresh owner readback still checks **52 helpers and 104 snapshots**. Metadata
readback still replays **136 syscalls**, **124 complete filesystem/table
transitions**, **44 stat outputs** and **44 actual clock returns**. All **177
model/reader tests** and [eighteen offline controls](checks/controls/controls.json)
pass. Controls alter saved data only, including missing invalidations/releases,
wrong permissions/addresses/CPU/modes, false barriers, missing root handoffs and
PCID-enabled handoff claims. No altered guest state or third-party program runs.

The history retains four distinct failed attempts with exact selected source
versions, native commands and raw outputs:

- `per-page-breakpoints`: the full run exceeded 600 seconds, with zero resource
  events. It provided no complete enclosing-application result.
- `global-nochain`: CPU logging with global chaining disabled also exceeded
  600 seconds. Its two completed publications and progress file remain explicitly
  incomplete. Later success does not itself establish either timeout's cause.
- `cancelled-block-reader`: the reader counted one explicitly cancelled block
  attempt as an extra loop observation. The raw trace distinguishes the matching
  retry. The parser correction preserves unmarked-duplicate rejection.
- `detached-cleanup-reader`: all four application iterations passed, then the
  model incorrectly required the user root during final completed-process
  cleanup. The subsequent run adds actual root-handoff observations and the
  separately constrained detached-cleanup case. No exact active-publication
  snapshot was retained at that earlier failure.

The installed-version logging pilots stop after one and two ordinary initial
publications. Their records explicitly deny complete guest retirement. The
default-chaining pilot captures all **35** corresponding states; its first
saved reader rejected a CPU file containing only known TB-link diagnostics.
The original parser and corrected readback remain. A preliminary layout check
also looked for the publication IR in `runtime.ll`; the actual compiled block
is in `dispatch.ll`. No exact log/source snapshot is retained for that preliminary
lookup error. None of these failures is a reported AI safeguard denial.

The [input delta](inputs.json) adds eleven current source snapshots and one
first-party IR artifact over the preceding
[mapping record](../stage-3-mapping-aliases/README.md), yielding **192 selected
sources and 94 artifacts**. Earlier packed inputs remain inherited by identity.
No new third-party ELF or initramfs is added to Git. Preceding mapping and
thirteen-audit results retain their original scopes and are not claimed rerun.

[Frozen replay](replay.json) restores the merged inputs, reruns all 177 models,
and reproduces the exact current layout, publication, owner, metadata and
eighteen-control reports. It executes Python readers only.

The [initial frozen replay](history/first-frozen-replay/status.json) omitted the
original memory IR source from its selected bundle. That source is now retained
with its exact native-profile digest; the initial input list and failed reader
log remain. This is a replay-input omission, with no guest/compiler execution.

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-translation-publication/replay.py linux-kernel/.cache/x86_64/translation-publication-replay-new
```

Use a new output directory. [Archive validation](validate-archive.py) verifies
the evidence delta, inherited packed identities, failure scopes, pilot records
and replay. Hashes bind selected bytes; they do not authenticate observations or
construct kernel authority. [Review](review.json) treats this as high risk, using
single-agent authority/source/diff review, source/native/GNU/log interpretations
and model/data checks. The observer and readback share model/parser code; no
independent human security review is claimed.

This qualifies the selected **local heap publication and teardown** evidence.
Other mapping editors, continuous mode and ownership lifetime, competing AP
actions, DMA and all asynchronous entries remain unqualified. Other callers and
private-counterpart loans, complete exception/fatal/whole-syscall effects, unrun
native lengths/refusals, all fallback gates and canonical clean production
reconstruction/activation remain required. The current record is not a complete
compiler/VM producer or a new Rust qualification. Core-language meaning and
executed kernel behavior are unchanged. Stage 3, initial Step 4 and full Linux
distribution, Docker/Podman and VM-host acceptance remain open.

Primary facts: [Intel SDM Volume 3A, 253668-083US](https://cdrdv2-public.intel.com/819714/253668-sdm-vol-3a.pdf),
sections 4.10.4.1–4.10.5; [QEMU invocation](https://www.qemu.org/docs/master/system/invocation.html);
upstream QEMU 10.2.2 [INVLPG translation](https://github.com/qemu/qemu/blob/v10.2.2/target/i386/tcg/translate.c#L3231)
and [execution logging/cancelled blocks](https://github.com/qemu/qemu/blob/v10.2.2/accel/tcg/cpu-exec.c#L246).
Installed tools are QEMU **10.2.2**, GDB **17.2** and GNU objdump **2.46.1**;
the pilots provide installed-version evidence separately from consulted source.
