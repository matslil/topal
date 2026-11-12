# Stage 3: native fixed pipe output observations

`TK-DEC-283`, `TOPAL-REQ-SYSTEMS-USER-EXTENT-001`,
`TOPAL-COMP-SYSTEMS-USER-EXTENT-001`, `TOPAL-SYSTEMS-USER-EXTENT-001`.

The [read-only observer](../../../../labs/qemu/x86_64/user_extent_pipe_fixed_observer.py)
adds ordinary native admission/store/readback observations to the preceding
[conditional fixed-output audit](../stage-3-fixed-pipe-audit/README.md).
It uses the unchanged source-built pipe fixture. No application instructions,
guest registers, mappings, ownership records or user data are injected.
The group runs only when explicitly selected; existing default observer groups
remain enabled. One breakpoint explicitly orders store completion before
readback start when they share the same PC.

The [fresh VM command](native/run.json) passes with all five changed source
files stable throughout. The Rust case takes 227.26 seconds, and its command
takes 227.45 seconds. Validation swap is zero, peak memory is 477806592 bytes,
and all recorded memory-event counters are zero. Four ordinary fixture
executions issue 144 syscalls. The selected results are:

| Output | Attempts | Completed | Refused | Store/readback observations |
| --- | ---: | ---: | ---: | ---: |
| Eight-byte descriptor pair | 24 | 4 | 20 | 8 |
| Four-byte readable count | 40 | 20 | 20 | 20 |

Each observed helper result preserves its specified registers and captured
arguments. Refusals match the source syscall's actual candidate, `EFAULT` and
unchanged phase. Accepted transfers preserve R8/R9, exact payload/readback
values and user-writable translations at each observed access boundary.
Observed candidate physical ranges exclude selected supervisor table, pipe,
context and stack ranges, and their translation tables. Those exclusions and
equal boundary observations **do not establish complete ownership, nonaliasing
or continuous mapping lifetime**. Existing complete selected FIFO/table/window
and terminal-retirement assertions still pass.

[Independent saved-record readback](native-readback.json) matches all 24
completed transfers, 40 refusals and 28 accesses to source syscall order. It
reconstructs readable-count payloads from queue progress and checks pair
payloads, translations and exclusions. Eight compared artifacts, including
kernel/provider/census/graph/origin/audit, installed first-party application and
boot resource, match the preceding checkpoint exactly. The raw source-link ELF
differs only in three nonallocated debug sections; all three allocated sections
are identical. The [section comparison](source-link-section-difference.json)
binds both raw links. The [first-reach record](native-coverage.json) finds 79
sites in this run, all within the preceding matching-kernel 88/91 union.

The [recomputed conditional index](caller-access-index.json) joins six reports:
**81/91 helper sites and 309 user-access instructions**, with ten sites still
without selected intervals. These are local conditional contracts, not complete
caller qualification. All three fallback obligations remain open.

The [source identities](source-identity.json) retain 141 selected inputs: seven
local snapshots and 134 inherited snapshots. Five are the changed observer/test
sources; two explicitly retain the unchanged model and its tests.
[Fourteen selected artifacts](artifact-identity.json) include ten inherited
snapshots and four local snapshots. The prior first-party source link is
retained separately for the debug-section comparison. No pinned third-party
application binary is copied or changed.

[Frozen replay](replay.json) reproduces all six static reports, all 69
model/reader tests, native-record correspondence and raw-link section comparison
without executing an ELF file, compiler, GNU decoder, debugger or VM:

```sh
scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-fixed-pipe-native/replay-fixed-pipe-native.py linux-kernel/.cache/x86_64/fixed-pipe-native-replay-new
```

Use a new output directory. Equal historical baseline bytes are restored from
their recorded identities; replay is neither a second native execution nor an
independent clean producer. The [review](review.json) retains the failed raw-link
packaging assumption and two incomplete replay-input attempts; their original
statuses remain failures. Final replay includes both missing unchanged model
files and requires all 69 tests. No model/provider safety notice occurred.

Risk remains high if this evidence is overstated. Review uses one agent's
authority/source review, actual-observer counterexamples, ordinary syscall
execution, separate source-order/payload/translation readback and frozen replay.
Kernel code and authoritative language/system meaning are unchanged. Complete
actual owners/nonaliasing, continuous lifetime, helper/entry/callee/return/fatal/
exception effects, ten remaining sites, fallbacks and canonical production
qualification remain required. Stage 3, initial Step 4 and the original kernel
objective remain unfinished.
