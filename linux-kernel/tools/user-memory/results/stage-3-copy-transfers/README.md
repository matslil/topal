# Stage 3 REP admission, dataflow and ordinary copy observations

`TOPAL-SYSTEMS-USER-EXTENT-001`, `TK-DEC-283`.

The [static audit](copy-transfer-audit.json) follows 15 actual admission paths
to 14 REP copy instructions. It interprets a restricted raw x86 subset to
track candidates, aliases and complete byte counts across helper calls. It
requires CLD, exact admission/copy byte-count equality, no scaled-count wrap,
and accounted incoming edges. Removing each complete admission group's success
edges makes its copy unreachable from the listed direct-control roots. Original
root closure, instruction bytes, edges and actual LSTAR/exported roots are
checked first by the preceding graph validator.

The paths cover five command-buffer outputs, signal action/mask input/output,
ordinary and named directory output, uname, clock time/resolution output and
sleep-interval input. `clock_gettime` and `clock_getres` share one copy. Their
two success edges must jointly cover that copy; claiming individual dominance
would be false. The clock-time path includes a 46-instruction sampler contract:
raw register effects preserve RBX/R12, and the wrapper/leaf's nonrecursive call
shape returns without an unbalanced stack change. Its clock owner, MMIO effects,
entry/exception contracts and mapping lifetime remain separate obligations.

The audit exposes counterpart expressions and clock staging stores instead of
granting ownership because an operand uses R12 or points into linked code.
Distinct symbolic expressions are not a nonaliasing proof. Shared-helper
soundness/preservation, captured-entry stability, counterpart bounds/ownership,
complete roots/callees/exceptions and stable mappings remain prerequisites.
These intervals do not establish complete syscall effect/error ordering or
classification of the other addressed/string instructions in the kernel.

The read-only GDB observer records every selected call's actual R8/R9 and
carry result, checks all eleven helper-preserved registers, the erased return
word, RSP and captured arguments, then pairs REP start with completion. It
checks actual endpoints, complete count, DF, RCX exhaustion, pointer advances
and exact copied bytes. All covered user and counterpart pages have their
actual present/user/write permissions checked. Counterparts must be supervisor
mappings; observed physical source/destination ranges cannot overlap each other
or the table pages on their observed translation paths. The boundary checks
compare CR0/CR3/CR4 and translation words with hardware A/D changes masked.
They do not prove continuous stability, complete table/backing ownership or
absence of aliases with every other translation/storage owner.

| Final cohort | Admission attempts | Refused | Completed copies | Copied bytes | Completed paths |
| --- | ---: | ---: | ---: | ---: | ---: |
| [Pinned BusyBox exit](application/native-run.json) | 5 | 0 | 5 | 339 | 5 |
| [Ordinary process/poll](poll/native-run.json) | 288 | 118 | 170 | 12325 | 14 |
| [Ordinary storage](storage/native-run.json) | 284 | 118 | 166 | 9397 | 15 |

The storage cohort alone completes every selected path. Its named-directory
path has 24 attempts, twenty refusals and four copies totaling 192 bytes.
Poll retains its 120 source-issued calls and 1064 completed controller callbacks;
storage retains 208 source-issued calls and 1168 callbacks. The pinned run
retains 77 dispatch callbacks. Existing source/native state and terminal checks
pass. Refused selected admissions have no corresponding selected REP invocation;
this is not a claim about all effects of the enclosing syscall.

[Application](application/readback.json), [poll](poll/readback.json) and
[storage](storage/readback.json) readback each independently decode all 172
selected prelude/local/sampler instruction boundaries with GNU and walk backward
from all fourteen copies with their admission edges removed. Every compared
kernel object/image (17), pinned or source-built application, boot resource and
console byte matches its preceding mode checkpoint. The common graph/source
metadata is compared against the explicitly byte-identical kernel with the
preceding source index. Pinned third-party application bytes remain outside
Git. The two unchanged first-party source-built ELF images and their component
identities are retained with their cohorts.

Each cohort's `gnu-records.json.gz` contains every original GNU command/stdout/
stderr file with its name, byte length and SHA-256. Generated `inspect.gdb`
scripts are retained as `inspect.gdb.gz`. Packaging verifies exact decompression
equality before replacing the newly generated duplicate text files.

The original 91-call census remains unchanged. Native first reach remains 38
sites for pinned exit, 77 for poll and 81 for storage. Those add no new site to
the preceding 88-of-91 matching-kernel union and close none of the three fallback
obligations. Canonical production retains its legacy selection and checks.

[Six offline controls](controls.json) reject a missing shared clock caller,
lost helper alias, sampler clobber, missing CLD, count mismatch and pointer
mismatch. They fault-inject checker configuration/effects in a separate Python
process with unchanged ELF and metadata; no corrupted guest artifact is run.
Six unit cases cover scaled-count overflow, alias overwrites, unsupported
direction/address modes, exposed memory effects, protected sampler registers
and the actual observer's paired-breakpoint lifecycle. That last test stubs
translation/memory checking and proves scheduling behavior only. It is not
native memory-safety evidence.

The [first observer attempt](initial-observer/failure.json) failed because the
REP PC can be revisited between iterations. Disabling its start breakpoint
until completion fixed that issue. A [preceding pinned run](history/application-v2/scope.json)
passed, but [source poll](history/poll-v2/scope.json) then exposed a valid
null-input signal path reaching the post-copy address without executing REP.
The final observer enables completion only at start, disables it at completion
and re-enables start for the next invocation. The [storage attempt using that
earlier observer](history/storage-v2/scope.json) was explicitly stopped through
its confirmed own bounded scope; it has no completion/resource record and no
pass is claimed. Earlier source hashes, observer/test overrides, failed records
and guest outputs remain retained. These are observer/control-flow issues,
not kernel failures or AI safeguard notices.

The [initial readback](initial-readback/failure.json) named source-index metadata
that did not yet exist in the older poll build. Final readback uses the older
mode-specific executable/application/console baseline and a separately verified
byte-identical common-kernel metadata baseline. It does not fabricate an earlier
source-index record.

[Seven final post-checks](post-validation.json) pass: all 28 model/reader tests,
both source-fixture build tests, workspace and explicit included-file formatting,
locked workspace/all-target checking, the unchanged Rust size ceiling and
whitespace. Final selected tests take 90.86 seconds (pinned), 264.62 seconds
(poll), and 229.23 seconds (storage). Their peak cgroup memory is 391901184,
415207424 and 422862848 bytes; post-check peak is 561655808 bytes. All four final
bounded commands retain zero swap and zero limit/OOM/kill events. Source hashes
remain unchanged during each final native command. No deadline, reserve, guest
register/memory state, host protection or filesystem/network boundary is relaxed.

The [119-input selected record](source-identity.json) stores five changed/new
inputs and explicitly inherits 114 unchanged snapshots from the preceding
[scalar](../stage-3-fixed-transfers/README.md) and
[source-index](../stage-3-process-origins/README.md) archives. The
[artifact record](artifact-identity.json) similarly references unchanged exact
kernel/graph/source-index bytes. The [manifest](manifest.json) distinguishes
local files from inherited references. Keep these adjacent packages together.
[Python-only replay](replay.json) restores the exact selected inputs, rechecks
the whole source join and reproduces the complete REP audit:

```sh
rtk python3 linux-kernel/tools/user-memory/results/stage-3-copy-transfers/replay-copy-transfers.py linux-kernel/.cache/copy-transfer-replay
```

Replay does not rebuild a kernel or run LLVM, GNU, GDB or a VM. Selected inputs
are not the complete producer or a production cold reconstruction.

The underlying memory boundary remains high risk. Review is single-agent
authority/full-diff review, separate GNU/backward readback, fault controls and
ordinary VM observations; no independent human security review is claimed.
All other access classes, complete helper/owner/lifetime/counterpart and
entry/callee/exception contracts, native mode/stale-state qualification,
canonical binding and exact committed production reconstruction remain open.
Stage 3, initial Step 4 and the original full kernel goal are incomplete.
