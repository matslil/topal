# Stage 3: shared stat output preparation and native observations

`TK-DEC-283`, `TOPAL-REQ-SYSTEMS-USER-EXTENT-001`,
`TOPAL-COMP-SYSTEMS-USER-EXTENT-001`, `TOPAL-SYSTEMS-USER-EXTENT-001`.

The [conditional stat reader](../../audit_stat_outputs.py) checks descriptor and
namespace admission paths to one shared 144-byte copy. Exact candidate/count,
return-word retirement, carry gates, staging wipe, DF, restricted register and
memory effects, publication operands and the combined success-edge cut are
checked against the actual linked instructions and graph. The reader exposes
other memory effects rather than granting ownership to their numeric addresses.

Six actual callees have selected normal-return contracts. The metadata validator
contains 1140 instructions, saves eight registers in 64 bytes, restores them in
reverse order, and erases and checks the retired spill words. Its body cannot
write memory, change RSP or unsaved registers, or bypass restoration on a normal
return. Nested wrappers and lookup add a maximum 96-byte callee stack footprint,
including return words. These checks do not establish stack ownership, fault
behavior or complete entry/callee/exception contracts.

The [ordinary C fixture](../../../../labs/qemu/x86_64/user_extent_stat_cases.c)
executes four times in the [fresh stat VM run](native/stat/run.json): **152
syscalls and 88 stat queries**. Its source and thirteen selected changed inputs
remain stable throughout this and the other final runs. The results include:

| Path | Attempts | Completed copies | Refused output admissions |
| --- | ---: | ---: | ---: |
| Descriptor stat | 52 | 40 | 12 |
| Namespace stat | 28 | 20 | 8 |

Eight additional queries fail descriptor/path lookup before registered output
events. Successes cover terminal input/output, immutable files, root/bin/tmp
directories and a directory descriptor, both pipe endpoints, anonymous files,
and linked and unlinked volatile files. The fixture includes cross-page output,
read-only output refusal and invalid-buffer/error ordering.

The [read-only stat observer](../../../../labs/qemu/x86_64/user_extent_stat_observer.py)
checks all 144 output bytes against private-state-derived records, surrounding
output bytes, complete selected table/FIFO/filesystem regions and protected
context across each query. The copy observer checks helper/candidate/count and
captured-argument preservation, byte transport, and translations/control at the
observed boundaries. Equal boundary observations do not prove continuous
mapping lifetime or complete ownership/nonaliasing.

[Separate saved-record readback](native-readback.json) matches all sixty copies,
twenty refusals and eight early lookup errors to source syscall arguments and
results. It independently checks selected constants, padding, pipe equality and
volatile-file size/link/inode relations. Dynamic mode/time/inode values still
use the live observer's private-state oracle; this is not an independent
reconstruction of every private read or a second native execution.

[GNU readback](gnu/readback.json) agrees on 1384 instructions across six windows,
register/memory effects, eight saved-register names and reverse restoration,
and the reverse shared-copy admission cut. [Fourteen offline ELF controls](controls/controls.json)
reject altered candidate/count, copy, saved-register, restoration, retirement
and carry-gate artifacts, including a graph-consistent restoration bypass.
Those changed images are never executed, booted or loaded into a VM.

The existing [ordinary buffer regression](native/source/run.json), unchanged
[pinned BusyBox regression](native/pinned/run.json) and two
[fixture build tests](native/fixtures/run.json) also pass. All thirteen changed
inputs have the same frozen hashes across these four final commands. Validation
swap and every recorded memory-event counter are zero. Seventeen kernel objects
and images per native cohort match the preceding fixed-pipe checkpoint; the
default ordinary application's installed image and boot resource also match
their preceding baseline. [Artifact comparisons](artifact-comparison.json)
retain those identities. No pinned third-party binary is copied into this
archive or modified.

The [joined conditional index](caller-access-index.json) now contains **83/91
helper sites and 310 distinct user-access instructions**, leaving eight sites
without selected intervals. The [matching-kernel first-reach record](native-coverage.json)
retains the preceding 88/91 union; the fresh stat/source/pinned observations
reach subsets of that union. It does not relabel earlier native evidence.

[Selected source identities](source-identity.json) bind 147 inputs: thirteen
local snapshots and 134 inherited snapshots. [Sixteen selected artifacts](artifact-identity.json)
include the inherited byte-origin map needed for source-join replay. The
[frozen replay](replay.json) reproduces the source join, seven reports, **82
model/reader tests**, and saved native-record readback without executing a
compiler, ELF, GNU decoder, debugger or VM:

```sh
rtk proxy env RUST_MIN_STACK=16777216 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-stat-outputs/replay-stat-outputs.py linux-kernel/.cache/x86_64/stat-output-replay-new
```

Use a new output directory. This restores selected inputs, not a complete clean
producer. [Archive validation](validate-archive.py) checks recorded bytes,
references, current selected sources, report/index bindings and local links.

The [review record](review.json) retains the failed duplicate first-reach
bookkeeping run and its exact twelve input snapshots, the earlier narrower
passing fixture and thirteen snapshots, the failed GNU reader, and the first
replay's missing-origin-map log. Their original scopes and failures remain.
Coincident read-only first-reach observations now require identical records;
conflicting records still fail. No model/provider safeguard notice occurred.

Risk is high if conditional evidence is overstated. Review is by one agent,
with authority/source review, separate raw/GNU readers, malformed-artifact
controls, ordinary native regressions and frozen replay. Kernel executable bytes
and authoritative language/system meaning are unchanged. Actual private/table/
stack/backing/output owners, nonaliasing, continuous lifetime, complete helper/
entry/callee/return/fatal/exception contracts, eight remaining sites, all three
fallback obligations and canonical clean production reconstruction remain
required. Stage 3, initial Step 4 and the original kernel goal remain unfinished.
