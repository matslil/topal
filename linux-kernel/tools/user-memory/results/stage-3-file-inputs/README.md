# Stage 3: staged volatile file inputs

`TK-DEC-283`, `TOPAL-REQ-SYSTEMS-USER-EXTENT-001`,
`TOPAL-COMP-SYSTEMS-USER-EXTENT-001`, `TOPAL-SYSTEMS-USER-EXTENT-001`,
`TOPAL-SYSTEMS-VOLATILE-FILE-TRANSFER-001`,
`TOPAL-SYSTEMS-POSITIONAL-FILE-TRANSFER-001`.

The [conditional reader](../../audit_file_inputs.py) follows ordinary and
positional volatile-file input from readable admission to the first user-to-
staging `REP MOVSB`. It checks clipping to 256 bytes, the file-page remainder
and remaining file-position capacity, exact page lookup/allocation preparation,
clock success, saved candidate/count registers, the staging destination and
direction flag. Admission success dominates each selected copy. Later staging
publication and private page copies remain separate obligations.

The clock wrapper saves twelve registers and restores them in reverse order.
Its selected maximum stack cost is 112 bytes. The actual 269-instruction RTC
leaf has no nested call or stack-pointer operation; its exact table reload,
three staging wipes, seven-word BCD loop, other table writes, calendar reads and
port I/O are exposed. These expressions grant no owners. The reloaded process
table must equal the captured entry's table, and private storage must not alias
the live stack. **Erasure of the 96 retired spill bytes is unverified** and
remains an acceptance blocker.

Each selected user copy has one live saved `DI` word. The observer requires
`copy RSP = admission RSP - 8`, preserves that word across the copy and checks
its physical disjointness from source and destination. Existing copy cases
retain their original zero-word stack relationship. Runtime observations also
check the actual clipped count, captured table, payload, helper/captured-argument
preservation and equal boundary translations. Boundary equality does not prove
continuous translation stability or cleanup of retired clock spills.

The expanded [ordinary C fixture](../../../../labs/qemu/x86_64/user_extent_storage_cases.c)
passes four executions and **248 syscalls** in the [storage VM run](native/storage/run.json):

| Input path | Attempts | Completed | Refused | Bytes |
| --- | ---: | ---: | ---: | ---: |
| `write` | 20 | 8 | 12 | 1,088 |
| `pwrite` | 20 | 8 | 12 | 48 |

Each path exercises both resident and missing file pages four times. A 512-byte
write request returns the bounded 256-byte quantum. A positional 512-byte
request uses only four mapped input bytes before a file-page boundary despite
an inaccessible requested suffix. Reads verify the resulting contents and
holes; positional operations preserve the descriptor position. Earlier error,
EOF, refusal-without-effects and complete file-storage reclamation assertions
remain. Sixteen early zero/numeric results do not reach input admission.
The conservative source stack bound is 1320 of 2048 bytes, including the
168-byte storage frame cost with its return word.

[Independent saved-record readback](native-readback.json) reconciles forty
admissions, sixteen completed copies, twenty-four refusals and all 1136 copied
bytes with actual syscall arguments/results and deterministic payloads. This
is an independent interpretation of saved records, not another native run or
a transcript of every private read.

[GNU operand/effect readback](gnu-operands.json) checks 422 selected instructions,
including all 269 RTC instructions, twelve-register save/restore order, the
live saved word, copy setup and two reverse admission cuts. It reprojects the
[preceding full GNU transcript](../stage-3-pipe-loops/gnu-graph/gnu-verification.json.gz)
for the byte-identical kernel: 31,024 instructions and 36,950 graph edges.
This increment performs no new GNU decoder execution.
[Twenty-one offline ELF controls](controls/controls.json) reject altered
candidate/count/counterpart/cap, stack saves/restores and aliases, RTC table/BCD
effects, carry and page-lookup preservation after outer identities are rebound.
Changed control images are never executed, booted or loaded into a VM.

The [ordinary buffer regression](native/source/run.json), unchanged
[pinned BusyBox regression](native/pinned/run.json) and two
[fixture build tests](native/fixtures/run.json) pass with the same nine frozen
changed inputs as the storage run. All four commands record zero validation
swap and zero memory-event counters. The pinned third-party ELF and its boot
archive are not copied here. [Artifact comparisons](artifact-comparison.json)
preserve seventeen core objects/images and thirteen preceding metadata/reports
in each cohort, plus both default source application/boot images.
**No executable kernel bytes, core language or authoritative design meaning
change.** Owned production selection remains inactive.

The [conditional index](caller-access-index.json) now has **88/91 helper sites
and 316 distinct user-access instructions**. Three sites still lack selected
intervals: multi-variant warning input, the named-root fallback and random
output. [Fresh first-reach coverage](native-coverage.json) is 83/91 across these
three cohorts, or **87/91** when explicitly joined with the preceding three
cohorts on this same kernel. Earlier, different-kernel coverage is not relabeled.
First reach and conditional interval coverage are different measurements.

[Selected source identities](source-identity.json) bind 155 inputs: nine local
and 146 inherited snapshots. [Thirty-three selected artifacts](artifact-identity.json)
retain the executable, graph/origins, ten reports, unchanged components and the
first-party storage fixture. [Frozen replay](replay.json) exactly reproduces the
source join, ten reports, **100 model/reader tests**, native saved-record
readback and saved GNU operand/effect/cut readback:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-file-inputs/replay-file-inputs.py linux-kernel/.cache/x86_64/file-input-replay-new
```

Use a new output directory. Replay executes no ELF, compiler, GNU decoder,
GDB or VM. It is not a complete clean producer.
[Archive validation](validate-archive.py) checks selected frozen/current sources,
artifact/report/index/native bindings and links.

[Review](review.json) is by one agent with higher-authority/source/full-diff
review, separate GNU/record interpretation, counterexamples and native tests.
Risk remains high for incorrect memory or evidence claims. Two reader failures
are retained: a short wipe-prefix match also matched an `RDI` reset, and a GNU
comment was initially retained in a destination operand. Both are fixed in the
final readers; neither was a kernel assertion or model/provider safety notice.

Actual owners/nonaliasing, continuous check-to-copy lifetime, retired clock
spill cleanup, complete helper/entry/callee/return/fatal/exception/post-copy
effects, three remaining sites, fallback obligations, further native profiles
and canonical clean production reconstruction remain open. Stage 3, initial
Step 4 and the full Linux-compatible kernel goal remain unfinished.
