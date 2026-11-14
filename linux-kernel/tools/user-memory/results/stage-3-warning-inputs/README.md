# Stage 3: selector-bound shell warning inputs

`TK-DEC-283`, `TOPAL-REQ-SYSTEMS-USER-EXTENT-001`,
`TOPAL-COMP-SYSTEMS-USER-EXTENT-001`, `TOPAL-SYSTEMS-USER-EXTENT-001`,
`TOPAL-SYSTEMS-STATIC-ELF-SHELL-TERMINAL-WARNING-001`.

The [conditional reader](../../audit_warning_inputs.py) checks the shell warning's
shared readable admission and all three selector-dependent input regions. The
retained selector chooses lengths four, forty and one. The prelude checks the
requested count against that selected length, preserves `RDX` through the exact
inline stack/disposition checks and reloads the captured candidate before
admission. The result gate precedes every selected load. After admission, the
same retained selector chooses one four-byte load, five eight-byte loads or one
byte load. Every load lies inside its variant's exact admitted range.

The reader checks all selected instruction boundaries, predecessor closure and
nonvacuous success-edge dominance. Inline private reads remain owner/nonalias
obligations. The argument relies on shared helper preservation of `RSI`, `RDX`
and the retained selector at `GS:616`, and on stable entry/mapping ownership.
The seven selected loads do not establish complete warning publication,
register-staging cleanup, full caller/exception effects or production safety.

The [read-only observer](../../../../labs/qemu/x86_64/user_extent_warning_observer.py)
records actual descriptor/count/candidate/selector binding, helper register and
captured-argument preservation, retired helper return-word erasure, each scalar
load and complete staged register bytes. It compares translation/control and
complete referenced table bytes at the boundaries. It changes no guest state.
Boundary equality does not prove continuous translation stability or create
memory ownership.

The [storage VM run](native/storage/run.json) reaches all three ordered variants:

| Segment | Admitted bytes | Scalar loads |
| --- | ---: | ---: |
| `sh: ` | 4 | 1 |
| `can't access tty; job control turned off` | 40 | 5 |
| newline | 1 | 1 |

[Independent saved-record readback](native-readback.json) reconciles their
selector order, exact lengths, seven load operands and all 45 staged bytes with
the first-reach record and actual serial warning. Helper/mapping assertions
retain their native observer evidence. This is not another native execution or
refusal-path qualification by itself. The existing four source executions and 248 storage
syscalls retain their file-transfer, positional, refusal and reclamation checks.
Source stack bounds remain 1320 of 2048 bytes for storage.

The [ordinary-buffer saved-record join](source-native-readback.json) separately
reconciles fifteen admissions with actual source-built syscalls. Four invalid
candidates per segment return `EFAULT`, preserve phase and queue state and have
empty scalar-access records. The subsequent valid request for each segment
copies its exact payload. This adds twelve native refusals and three completed
copies totaling 45 bytes. Protected-state, helper and mapping assertions retain
the live observers' evidence; this does not reconstruct continuous stability or
actual owners independently.

The [ordinary buffer regression](native/source/run.json), unchanged
[pinned BusyBox regression](native/pinned/run.json) and two
[fixture build tests](native/fixtures/run.json) also pass. The pinned profile
records no warning execution, preserving its profile distinction. All four
commands use the same five frozen changed sources, zero validation swap and
zero memory-event counters. No pinned third-party ELF or boot archive is copied
here. [Actual artifact comparisons](artifact-comparison.json) preserve seventeen
core objects/images and fourteen preceding metadata/reports per cohort, plus
32 inherited artifacts. The first-party `source-user.elf` intermediate has
different nonallocated DWARF bytes in three debug sections. Its
[separate comparison](source-intermediate-comparison.json) verifies identical
ELF/program/section headers, all allocated sections and every load-segment byte.
The installed fixture ELF and boot archive remain identical. Executable kernel bytes, core language,
authoritative design meaning and canonical production selection are unchanged.

[Independent GNU readback](gnu-operands.json) checks 74 selected instructions:
selector/count/candidate operands, absence of `RDX` or selector writes in the
inline prelude, seven scalar operands and the reverse admission cut. It uses
the preceding, byte-identical kernel's full GNU transcript of 31,024 instructions
and 36,950 graph edges. This increment runs no new GNU decoder.
[Fifteen offline ELF controls](controls/controls.json) reject changed lengths,
count/candidate binding, selector reload/dispatch, carry polarity, scalar
base/width/offset and a coherently rebound wrong-variant edge. Changed images
are never executed, booted or loaded into a VM. All **104 model/reader tests**
pass, including wrong-selector/count, high-bit selector, changed-byte and
out-of-order observation counterexamples.

The [conditional index](caller-access-index.json) has **89/91 helper sites and
323 distinct user-access instructions**, leaving random output and the
named-root fallback without selected intervals. [First-reach coverage](native-coverage.json)
remains 83/91 in the three fresh cohorts and 87/91 with explicitly bound
preceding same-kernel cohorts. Native staging records do not manufacture
additional first-reach coverage or close the separate fallback obligations.

[Selected source identities](source-identity.json) bind 158 inputs, five local
and 153 inherited. [Thirty-five selected artifacts](artifact-identity.json)
retain 32 unchanged preceding artifacts, the current source intermediate, its
recorded prior comparison input and the warning report.
[Frozen replay](replay.json) exactly reproduces the source join, eleven reports,
104 model/reader tests, both saved native joins, GNU readback and exact intermediate comparison:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-warning-inputs/replay-warning-inputs.py linux-kernel/.cache/x86_64/warning-input-replay-new
```

Use a new output directory. Replay executes no ELF, compiler, GNU decoder,
GDB or VM; it is not a complete clean producer.
[Archive validation](validate-archive.py) checks frozen/current source, artifact,
report/index/native and link bindings. [Review](review.json) is by one agent
against higher-authority/source/full-diff evidence and separate GNU/native-record
interpretations. Risk remains high for incorrect memory or evidence claims.
The first archive attempt incorrectly required equal debug bytes; a premature
replay then lacked the unfinished artifact record. Both failures and readers
remain in history. No model/provider safeguard notice occurred.

Actual owners/nonaliasing, continuous lifetime, helper/selector preservation,
complete staging/output/callee/entry/return/fatal/exception effects, two remaining
sites, fallback obligations, the unverified 96 retired clock spill bytes,
further profiles and canonical clean production qualification remain open.
Stage 3, initial Step 4 and the original Linux-compatible kernel goal remain
unfinished.
