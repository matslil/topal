# Stage 3 preserved admission values through staging

`TOPAL-SYSTEMS-USER-EXTENT-001`, `TK-DEC-283`.

The experimental owned selection retains admitted R8/R9 values through console
staging and descriptor read/pread output. Serial output also retains its
admitted pointer; its distinct partial-count interval remains unqualified.
Predecessor emission is preserved. This is a backend dataflow change with no
core-language or public interface change, and no assertion that the preceding
serialized implementation was exploitable.

The [selected report](artifacts/staging-copy-audit.json.gz) adds six paths:
console input, descriptor read/pread output, current-directory output and
sparse-file read/pread output. Raw encodings establish candidate/count equality,
explicit maxima (256 bytes, or five for cwd), direction and DF. Every incoming
local edge is closed. Removing the selected success edge cuts each copy from
the listed direct-control roots. Console over-limit and empty exits must not
reach the helper call. A closed 101-byte page-lookup leaf preserves the sparse
candidate/count and bounds the selected slot. Its private association reads,
staging wipes/readbacks, image/page reads and cwd staging are explicitly separate
owner/alias obligations. Numeric addresses do not grant those owners.

The [complete body comparison](kernel-difference.json) accounts for six
nine-byte GS loads becoming three-byte register moves and every differing
PC-relative fixup, including unreachable emitted blocks. The process body is
231627 bytes, 36 fewer than before. All 162 other provider function bodies
and relocations remain unchanged. A separate experimental target
change checks private zero-filled RAM in complete eight-byte chunks with a byte
tail. It preserves every byte predicate and rejects unreviewed binding/geometry
changes. Its [invariant review](private-clean/private-clean-invariant-review.json)
restricts wider volatile reads to complete stable private ordinary RAM; this is
not suitable for MMIO or racing aliases. Three component tests pass 210947
comparisons over the exact old/new LLVM bodies, including every bit, unaligned
short lengths, actual buffer sizes, guards, unchanged contents and null/zero.
Both functions use eight stack bytes. [Actual target binding](private-clean/private-clean-target-binding.json)
matches the tested 67-byte function to the emitted target and linked kernel;
all 32 other target bodies/relocations and fourteen intermediate component/
provider/predecessor objects remain unchanged. The complete emission/source join
and composed comparison reconstruct every byte of the new linked process
function with all nineteen external relocations. These are byte/binding checks,
not complete memory-ownership qualification.

[GNU graph readback](graph-readback.json) agrees on 31016
nodes and 36938 edges, with explicit external/indirect
boundaries. Separate [GNU operand readback](gnu-readback.json) checks
218 instruction boundaries in 15 windows
and six reverse success-edge cuts. The complete GNU graph decoder transcript
and original selected-window command/output records are bundled with their
identities. [Eleven offline controls](controls.json)
first require a byte-identical positive report, then reject altered caps,
candidates, UART count retention, cwd definitions, lookup preservation, staging
size, carry polarity and redirected console exits. Redirect controls update
their actual graph closure before requiring the specific bound rejection.
Changed first-party artifacts are never executed, booted or loaded into a VM.
These controls do not authenticate source provenance.

The [ordinary source regression](source-readback.json) passes 812 syscalls,
including 397 expected `EFAULT` results. Four callbacks each make eighteen
requests. New requests test partial console input and valid page/directory
crossings beyond page 34, preserve input/adjacent guards and restore the heap.
Cross-page cwd outputs are checked against the actual directory identity;
all four new outputs are `/bin` with five returned bytes. The source component
has 8064 linked text bytes and a conservative
1784/2048-byte stack bound. Existing complete stat,
directory, shell, command-queue, register, prefix and retirement checks remain.
The 7798 console bytes match the preceding transcript after removing exactly
eight new 24-byte markers.

The six [native cohorts](native-summary.json) use matching new kernel/object/
structured-report bytes. Rows count source-adapter calls where available;
pinned application checks retain their distinct scope. Seconds include runner
overhead and are not the guest deadline.

| Cohort | Source calls | EFAULT | Elapsed seconds | Reached helper sites |
| --- | ---: | ---: | ---: | ---: |
| source | 812 | 397 | 245.71 | 18 |
| storage | 208 | 112 | 227.76 | 81 |
| serial | 64 | 20 | 227.33 | 76 |
| pipes | 144 | 76 | 225.80 | 79 |
| poll | 120 | 64 | 287.15 | 77 |
| pinned | — | — | 115.38 | 38 |

The source cohort observes twelve completed new copies and eight ordinary
console refusals; its four other new paths are not reached. Storage observes
all six paths, with 89 attempts, 37 copies,
52 refusals and 603 copied bytes across those six.
Default cohorts retain the preceding fifteen-path REP observer alongside these
six. Native checks preserve helper registers/captured arguments and compare
actual candidate/count/DF, complete payloads, user/supervisor permissions,
physical nonoverlap with counterpart/translation storage and boundary mapping/
control observations. Boundary equality is not continuous-lifetime proof.
The dedicated serial observer separately verifies actual UART transport.

The pinned BusyBox image remains unchanged; identity and native logs are
retained without copying the third-party image into this archive. The full
compiler library suite passes **383 tests with eight explicitly ignored focused
VM tests**. All eight ignored tests are separately executed: six above collect
the helper census, while the additional [exec](native-additional/exec/test.log)
and [signals](native-additional/signals/test.log) cases provide semantic
regressions without a helper census. Existing native component/fault controls in
the regular suite retain their actual profile scope; they are not all read-only
observations. [Eight post-checks](post-validation.json) pass 52 user-memory
model/reader tests and eight host-observer/watchdog tests, both source-fixture
build tests, workspace and explicit included-file formatting, locked workspace/
all-target checks, Rust source sizes and whitespace. Resource records retain
the bounded cgroup limits, zero validation swap and actual events. Host
protections and guest memory/filesystem/network boundaries remain unchanged.

The original 1800-second full shared-workload attempts remain failed timeouts,
including the new kernel attempt which completed 17664 checked helper events
before its deadline. Repeating four images exceeded that test budget while
checks continued to advance. The [reviewed host budget change](method/full-workload-budget/preflight.json)
retains the existing 900-second per-image allowance within a 3600-second total
and adds a 300-second output-inactivity guard. Output activity is not proof of
semantic progress; final native/model assertions still decide correctness.
Focused guest deadlines, all checks, workloads and memory/isolation limits
remain unchanged. Four deterministic guard tests pass. The successful regular
suite uses this explicitly changed method; it does not establish a pass or
performance claim under the old 1800-second deadline.
The [actual full-run input comparison](method/full-shared-input-comparison.json)
finds the kernel, application, initramfs, caller selection/sites/owners identical
to the earlier failed current-kernel attempt. The complete saved debugger
program also agrees after normalizing only exact build paths and its one
loopback debugger port. The changed deadline/watchdog is in the host driver.
The successful full test retains its own first-party fixture, debugger/serial
logs, observations and every compressed full extent record. A separate
[streaming readback](full-shared-record-readback.json) binds all 19383 complete
records to their summary hashes and unchanged other fields, verifies gzip completion,
and rechecks compared input/program identities. This is correspondence
evidence, not another native run or a new ownership/permission proof.

Fresh debugger memory reads use bulk little-endian word decoding without
caching. Four decoder tests compare the predecessor result, changed-memory
rereads and fail-closed short reads. The staging reader also rejects an
unbounded graph root entering its backward count slice, including roots that
already have predecessors. Synthetic root controls and actual positive-report
equality are recorded separately; no actual kernel root was found in that
count slice.

Source identities distinguish the actual input versions. The six census runs
capture the same 19 modified inputs before the host budget change. Two saved
baseline harness inputs are retained separately and agree with the base Git
commit; they were not part of those per-run modified-input hash captures. The
regular suite, post-checks and additional exec/signal runs capture the current
22 modified inputs. Current selected-source identity includes 135 paths: 22
local snapshots and 113 inherited unchanged snapshots. Every accepted run passes
its actual unchanged before/after capture. This is selected-input provenance,
not a complete clean-checkout producer reconstruction.

Retained [failure records](failures/) remain distinct: an overly literal clock
displacement reader failed before boot; the first cwd fixture incorrectly
required `/` when the actual result was `/bin`; the initial broad compiler run
hit temporary-disk quota errors (its retained summary is not a full transcript);
and a native pinned test passed but its wrapper rejected changing reader input.
The latter is not source-freeze acceptance. Initial independent-reader GNU
format and altered-graph closure corrections were diagnostic-tool failures.
The decoder-only and original-budget shared retries, poll retries, initial
private binding-pattern rejection and zero-test filter mistake also remain
distinct failures or non-acceptance records. The zero-test command established
compilation only; the actual subsequent three-test component run is separate.
An initial [archive packaging attempt](failures/archive-baseline-metadata/archive.log)
selected an older equal-kernel directory without graph metadata. The corrected
baseline matches the preceding frozen kernel, provider, graph and census; this
was a packaging failure, separate from the passed VM tests.
None establishes a hidden AI safeguard decision.

The [current index](caller-access-index.json) identifies
79/91 helper sites with selected local
intervals and 306 user-access
instructions; twelve sites remain listed without intervals. Current matching
native execution reaches 88/91 sites.
Historical old-kernel execution is not relabeled. All three fallback obligations
remain open. No selected interval qualifies every other access in that caller.

The [135-input identity](source-identity.json), [artifacts](artifact-identity.json)
and [manifest](manifest.json) retain current sources, new kernel metadata,
earlier reader inputs, native versions, inherited Git-equal references and local
records. [Replay](replay.json) restores selected sources/artifacts and reproduces
the complete emission/source join and all five reports exactly. It also
replays the six-provider-edit comparison, actual private-predicate binding and
composed linked-body reconstruction against saved baseline, intermediate and
current first-party artifact sets:

```sh
rtk proxy scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-staging-copies/replay-staging-copies.py linux-kernel/.cache/staging-copy-replay
```

Replay does not rebuild or run a kernel, GNU/LLVM, GDB or a VM. [Review](review.json)
records high-risk, single-agent authority/implementation/diff review with separate
decoding, controls and native evidence; no independent human security review is
claimed. Complete helper/root/callee/exception contracts, actual table/backing/
counterpart owners, continuous entry-to-last-access stability, other accesses,
whole-syscall effects, native mode/stale-state qualification, canonical binding
and exact committed production/cold reconstruction remain required. Stage 3,
initial Step 4 and the full kernel/distribution/device/container/VM-host goal
remain incomplete.
