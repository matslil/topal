# Stage 2 ordinary functional checkpoint

The [record](qualification.json) freezes 2157 worktree source inputs and passes
eight commands: formatting, explicit included-file formatting, Rust file size,
locked workspace/all-target checking, seventeen model tests, three selected
compiler tests, workspace doc tests and whitespace. This is selected execution,
not a passing full-workspace run or completed Stage 2.

The current native guest completes 324 extent checks and 80 actual memory
operations, using real root/table/image/stack/heap owners. Native target
compilation and actual root-owner derivation also pass. Deliberate guest-state
controls were not executed in this checkpoint, following the current user
instruction to use ordinary development and functional testing.

[Independent readback](independent-readback.json) verifies source/log identities,
decodes 37 root allocation/pin/release calls before process entry, derives all
four owner bindings, parses linked ELF loads against the boot image and parses
the two boot-resource members. A separate flat permission/ownership calculation
matches all 324 native results. The complete probe/bridge/gateway path is 2272
bytes within 4096. Actual guest initialization changes CR0 from 0x80000011 to
0x80010011 while preserving every other bit. The native callback additionally
keeps runtime heap allocation identities distinct from all compiler-bound root
allocation slots, including retained translation storage.

The [replay](replay.json) restores archived first-party VM inputs and frozen
observer/model/manifest files into a fresh directory, then repeats the ordinary
324-check/80-operation VM successfully. It needs QEMU, GDB and the pinned firmware;
it does not need the original private VM cache or external BusyBox executable.

```sh
scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-2/replay.py linux-kernel/.cache/owned-extent-replay --run
```

Use a destination that does not exist. The helper validates both compressed and
uncompressed hashes before restoring. [Supporting programs](supporting-programs.json)
retain the separate reader; it expects a checkout matching the qualified source
hashes. [Tool information](supporting-tools.json) is supplementary readback after
the cohort, not a claim of a captured clean host/toolchain environment. Original
qualification executed from a frozen worktree, not a clean checkout.

The [first workspace attempt](../stage-2-initial-interrupt-failure/README.md)
retains a failed old interrupt-negative debugger assertion and four successful
isolated exact-image repeats; its cause is unresolved. Its 8243 guarded host
calls, 8192 permission combinations and 21 isolated guest controls belong to the
earlier target revision. Three source inputs changed afterward; those controls
are not claimed as fresh verification of this checkpoint. A subsequent serial
workspace run was [stopped for static review](../stage-2-review-stop/review-stop.json),
with its incomplete status preserved. No successful full-workspace result is
inferred from either attempt.

Functional-cohort peak was 2203799552 bytes, with zero limit/OOM/kill events and zero
validation swap. The wrapper and host reserve were unchanged. All-caller
migration, full production/source/native/boot qualification and original
distribution/container/VM-host acceptance remain unfinished. This checkpoint
does not replace those requirements or activate canonical publication.
