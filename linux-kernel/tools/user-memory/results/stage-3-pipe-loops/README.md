# Stage 3: guarded FIFO byte loops

`TK-DEC-283`, `TOPAL-REQ-SYSTEMS-USER-EXTENT-001`,
`TOPAL-COMP-SYSTEMS-USER-EXTENT-001`, `TOPAL-SYSTEMS-USER-EXTENT-001`,
`TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-TRANSFER-001`.

The experimental owned-memory selection now explicitly checks `1 <= R9 <= 4096`
immediately before each FIFO transfer admission. The loops execute a byte body
before testing their remaining count. These checks re-establish the required
positive bounded quantum after clock/description lookup and clipping. An
inconsistent trusted count is fatal; ordinary zero requests, empty/wait/EOF,
numeric range admission, atomic small writes and valid short transfers retain
their existing paths. Predecessor emission is unchanged and owned production
selection remains inactive. No core-language or authoritative design meaning
changes.

The [conditional reader](../../audit_pipe_loops.py) checks both exact guard
sequences, helper direction and retirement/carry gates, complete local loop
instructions, user cursor/count relationships and circular indexing. Guard and
admission success edges dominate the selected loops. Three user-access
instructions are covered: the read's store/readback and the write's load. Other
ring/object expressions remain private owner, index and nonalias obligations.

At each iteration, `0 <= completed < admitted <= 4096`,
`RCX = admitted-completed`, and the user cursor is `candidate+completed`.
The cursor advances one byte, the count decreases once, and ring progress is
masked by 4095. All user offsets fit the exact nonwrapping admission. Actual
private ring/index invariants, helper preservation and lifetime still need
their complete caller qualification; numeric expressions do not establish them.

[Whole-body reconstruction](emission-comparison.json) accounts for all 52 new
process bytes as two exact guard insertions and 219 derived relative-displacement
changes. Shifted relocation identities agree. Fourteen component objects,
including the predecessor provider/gateway, remain byte-identical. This is a
comparison against the recorded prior provider, not a clean independent build.
The [complete provider-function comparison](provider-function-comparison.json)
finds that the other 162 of 163 defined function bodies are unchanged.

[GNU whole-graph readback](gnu-graph/gnu-verification.json.gz) independently
decodes all **31,024 selected instructions and 36,950 graph edges**, including
their actual root closure, eighteen external transfers, six indirect sites and
267 terminals. [Selected GNU operand readback](gnu-operands.json) checks 58
guard/preparation/loop instructions and six reverse guard/admission cuts.
[Seventeen offline ELF controls](controls/controls.json) reject altered count
guards, candidate/count, strides, decrement, ring mask, erasure, readback and
carry artifacts after outer identities are rebound. Changed control images are
never executed, booted or loaded into a VM.

The expanded [ordinary C fixture](../../../../labs/qemu/x86_64/user_extent_pipe_cases.c)
passes four executions and **176 syscalls** in the [fresh pipe VM run](native/pipes/run.json):

| Loop | Attempts | Completed | Refused | Bytes |
| --- | ---: | ---: | ---: | ---: |
| FIFO read | 36 | 20 | 16 | 32,904 |
| FIFO write | 32 | 20 | 12 | 32,904 |

Sixteen completed transfers wrap the ring; full 4096-byte transfers and eight
zero-length calls are included. Earlier cross-page/read-only/refusal cases,
short reads, EOF/EAGAIN, fixed outputs and final FIFO/table retirement remain.
The C frame costs 208 bytes plus its return word; the complete conservative
source stack bound remains 1368 of 2048 bytes.

The [read-only loop observer](../../../../labs/qemu/x86_64/user_extent_pipe_loop_observer.py)
checks actual admitted counts/candidates, helper preservation and syscall
association. At loop entry/completion it checks complete user payload and ring
effects, consumed-byte erasure, register/captured-argument preservation, selected
physical exclusions and equal translations/control. It does not produce a
per-byte native transcript or prove continuous mapping stability.

[Independent saved-record readback](native-readback.json) reconciles forty loop
records and twenty-eight refusals with source syscall arguments/results. A
separate FIFO queue/cursor model verifies byte order, deterministic full-ring
patterns, exact short counts, all sixteen wrap facts and eight unused zero
requests. Runtime complete ring/erasure and boundary-translation assertions
remain the live observer's evidence, not a second native execution.

The [ordinary buffer regression](native/source/run.json), unchanged
[pinned BusyBox regression](native/pinned/run.json), three
[provider tests](native/provider/run.json) and two
[fixture build tests](native/fixtures/run.json) also pass. All five commands use
the same nine frozen changed inputs, with zero validation swap and zero recorded
memory-event counters. All three native cohorts have the same new kernel,
provider and nine reports. Pinned third-party application bytes are unchanged
and are not copied into this archive.

The [new-kernel conditional index](caller-access-index.json) has **86/91 helper
sites and 314 distinct user-access instructions**, with five sites lacking
intervals. The [fresh first-reach union](native-coverage.json) is **81/91** across
these three current-kernel cohorts. Earlier kernel observations retain their
original identities and scopes; its 88-site union is not relabeled as current
evidence. Further profiles on this changed kernel remain required.

[Selected source identities](source-identity.json) bind 152 inputs: nine local
and 143 inherited snapshots. [Thirty-five selected artifacts](artifact-identity.json)
include the current executable/graph/origins, nine reports, first-party fixture,
fourteen unchanged components and three prior-body comparison inputs.
[Frozen replay](replay.json) reproduces the source join, all nine reports,
**94 model/reader tests**, saved native FIFO records, complete body comparison
and saved GNU operand/cut readback:

```sh
rtk proxy env RUST_MIN_STACK=16777216 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-pipe-loops/replay-pipe-loops.py linux-kernel/.cache/x86_64/pipe-loop-replay-new
```

Use a new output directory. Replay executes no ELF, compiler, GNU decoder, GDB
or VM. Equal historical component bytes are restored using their recorded equal
hashes; this is neither an independent baseline producer nor a complete clean
producer. [Archive validation](validate-archive.py) checks selected frozen/current
sources, artifact/report/index bindings and links.

[Review](review.json) is by one agent, with higher-authority/source/full-diff
review, complete-byte reconstruction, separate GNU decoding, counterexamples,
native execution and saved-record models. Risk remains high for incorrect memory
or evidence claims. The first frozen replay failed because an inherited identity
row lacked an optional compressed hash; its reader and failure metadata remain
in history. Uncompressed source hashes and inherited-record binding remain
mandatory. No model/provider safeguard notice occurred.

Actual owners/nonaliasing, continuous check-to-copy lifetime, complete helper/
entry/callee/return/fatal/exception effects, five remaining sites, all three
fallback obligations, further current-kernel profiles and canonical clean
production reconstruction remain open. Stage 3, initial Step 4 and the original
Linux-compatible kernel goal remain unfinished.
