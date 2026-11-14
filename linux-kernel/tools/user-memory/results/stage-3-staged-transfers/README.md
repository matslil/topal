# Stage 3 finite loops and unrolled staged transfers

`TOPAL-SYSTEMS-USER-EXTENT-001`, `TK-DEC-283`.

The [summary](staged-summary.json) covers sixteen additional admission/transfer
regions and 208 distinct user-access instructions: nine finite indexed byte
loops and seven unrolled native-result copies. The complete raw-node/source
record is compressed under [its exact identity](staged-audit-identity.json).
Generated kernel bytes and canonical production selection are unchanged.

| Form | Regions | User-access instructions | Evidence |
| --- | ---: | ---: | --- |
| Shell uname output loop | 1 | 1 | RDI candidate retained, R9 zeroed, unit increment, exact 390-byte bound, linked source expression |
| Staged message/banner input loops | 8 | 8 | RSI candidate retained, R10 zeroed, unit increment, exact admitted bounds, GS staging stores |
| Native stat output/readback | 5 | 180 | Eighteen complete qword stores followed by eighteen matching readbacks per result; exact 144-byte admission |
| Native directory output | 2 | 19 | Nine/ten complete qword stores from staging; exact 72/80-byte admission |

The audit checks exact helper binding, candidate/count setup, immediate carry
refusal, the complete index/scalar byte ranges and source-to-output/readback
relationships. It validates preparation control flow, including alternate stat
payload constructions and captured-pointer reloads. Preparation cannot overwrite
the captured candidate or reload it from a different field. Register effects
outside the admitted subset fail closed. Removing each success edge makes its
entire selected interval unreachable from the verified direct-control roots;
all real incoming local edges must be accounted for, including loop backedges.

These are conditional local checks. GS and linked-address expressions do not
create owner authority. Candidate reloads rely on the captured-entry/GS binding
contract, and the shared helper, full entry/callee/exception contracts,
counterpart/table/backing ownership and continuous check-to-copy lifetime still
require complete qualification. Other accesses may remain before or after
every selected interval. No syscall-wide safety, error/effect ordering or full
kernel acceptance follows from these local bounds.

The paired read-only observer checks each selected call's actual R8/R9 and carry
result, all eleven helper-preserved registers, erased return word, RSP and
captured arguments. At the transfer boundaries it requires actual GS base to
equal the root-bound context, the user candidate to remain equal to the admitted
value, finite-loop indices to begin at zero and finish at the exact length,
and all copied bytes to match. It checks all covered user/counterpart mappings,
effective permissions, observed physical source/destination nonoverlap and no
overlap with the table pages on those translation paths. CR0/CR3/CR4 and boundary
translation observations agree with hardware A/D changes allowed. The observer
disables the loop-entry breakpoint until completion and arms completion only
after entry, preserving ordinary loop execution and valid bypasses.

| Fresh cohort | Selected attempts / completed | Refused | Bytes | Completed regions |
| --- | ---: | ---: | ---: | ---: |
| [Ordinary storage](storage/native-run.json) | 20 / 20 | 0 | 1910 | 16 |
| [Unchanged pinned BusyBox](application/native-run.json) | 18 / 18 | 0 | 1657 | 14 |

Storage executes every selected region. Pinned BusyBox does not execute the
clean profile's banner and profile-file stat paths. The new regions observe
**no refused admission** in these cohorts; native invalid-buffer/refusal cases
for these specific callers remain required. Offline malformed-artifact checks
do not discharge that obligation. Boundary observations likewise do not prove
continuous stability, every mapping owner, every alias or every possible path.

[Storage](storage/readback.json) and [application](application/readback.json)
readback each independently decode all 938 selected prelude/preparation/transfer
instruction boundaries with GNU, check 110 write/98 read operands and walk
backward from each of the 208 accesses with its success edge removed. Separate
finite-loop enumeration agrees on every accessed index. Selected GS displacements
are positive signed values; other displacement/address formats are not qualified
by this record. All seventeen compared kernel objects/images, pinned or
source-built application bytes, boot resources, original census/graph/source
metadata and console bytes match their preceding mode checkpoints. Original
GNU commands/stdout/stderr are losslessly bundled with per-entry identities.

The existing REP observations also pass: 166 completed copies in storage and
five in pinned exit. Callback returns remain 1168 and 77; first reach remains
81 and 38 helper sites. The prior matching-kernel union stays 88 of 91. Named
condition remains outside the listed direct graph, and all three fallback
obligations remain open. No new execution root is inferred from a label.

[Six offline controls](controls.json) corrupt copied first-party provider and
linked artifacts and rebind the outer census/graph/join identities, reaching
the semantic checks for loop limit, increment, candidate base, final stat-word
offset, captured-pointer reload and carry polarity. All six are rejected.
Corrupted artifacts are never compiled, booted, loaded into a VM or used to
change guest state. This is not source-provenance authentication. Pinned
third-party binaries remain unchanged and outside Git.

Six new unit cases cover body-first loop limits, GS effects without owner facts,
unsupported address forms, captured-pointer overwrite/reload, hidden candidate
arithmetic/calls and the actual observer's paired boundaries/final index.
The scheduling test stubs memory/translation and proves scheduling only.
A [development note](development-note.json) records the initial missing GS MOV
store classification and its correction before unit/native qualification;
it was not a kernel failure or AI safeguard notice.

[Seven post-checks](post-validation.json) pass: all 34 model/reader tests,
both source-fixture build tests, workspace and explicit included-file formatting,
locked workspace/all-target checking, unchanged Rust file-size ceiling and
whitespace. Selected tests take 271.88 seconds (storage) and 105.33 seconds
(pinned exit). The storage command also includes 5.16 seconds compilation.
Native peak memory is 1254060032 / 486318080 bytes, post-check peak
812933120 bytes. All three bounded commands retain zero swap and zero
limit/OOM/kill events; native source hashes remain unchanged during execution.
Deadlines, reserves, host protections and guest filesystem/network boundaries
are unchanged. No guest register or memory state is assigned by the observer.

The [caller/access index](caller-access-index.json) joins this record with the
preceding [scalar](../stage-3-fixed-transfers/README.md) and
[REP](../stage-3-copy-transfers/README.md) records: forty of 91 helper call sites
have selected local intervals covering 249 distinct user-access instructions;
the other 51 sites are listed explicitly. This is not complete caller coverage
or a global classification of all 14584 potential-access instructions. Its
[prior audit identities](prior-access-audits.json) match the fresh generated
reports and preserve their separate scopes.

The [122-input record](source-identity.json) stores five changed/new inputs and
inherits 117 unchanged snapshots. [Kernel/graph artifacts](artifact-identity.json),
the source-built image and preceding audit records are also explicitly inherited
after equality checks. The [manifest](manifest.json) separates local evidence
from inherited references. Keep the adjacent packages together. Exact
[Python-only replay](replay.json) restores the selected source/artifact bytes,
rechecks the complete emission-source join and reproduces the staged report:

```sh
rtk python3 linux-kernel/tools/user-memory/results/stage-3-staged-transfers/replay-staged-transfers.py linux-kernel/.cache/staged-transfer-replay
```

Replay does not rebuild a kernel or run LLVM, GNU, GDB or a VM. Selected inputs
are not the complete producer or a production cold reconstruction. Risk remains
high at the memory boundary. Review is single-agent authority/full-diff review,
separate GNU/backward/range readback, offline controls and ordinary native
observations, not independent human security review. Other access classes,
native invalid-buffer cases, complete owners/lifetime/helper/root/callee/
exception contracts, mode/stale-state qualification, canonical binding and
exact committed production reconstruction remain required. Stage 3, initial
Step 4 and the original kernel goal remain incomplete.
