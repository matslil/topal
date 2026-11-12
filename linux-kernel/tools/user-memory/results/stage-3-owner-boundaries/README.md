# Stage 3: actual owner observations at metadata helper boundaries

`TK-DEC-283`, `TOPAL-SYSTEMS-USER-EXTENT-001`.

The [read-only observer](../../../../labs/qemu/x86_64/user_extent_owner_observer.py)
adds actual allocation evidence around the successful stat and positional-write
admissions of the existing ordinary metadata program. It reads each selected
root token, live registry record, complete allocation cells, runtime heap token,
six translation pages, captured entry identity and paging controls at both helper
entry and return. Runtime contents are never cached between these boundaries.
Only immutable ELF symbol addresses are retained.

The [saved-record model](../../owner_snapshot.py) requires the selected issuer,
issued identity, exact allocation count/pins, registry state/provenance and every
consecutive physical cell. Complete selected allocation spans must be physically
disjoint, and lie outside all linked kernel PT_LOAD memory spans. Runtime heap
identity is distinct from every compiler-bound root identity. Actual GS/context,
CR3, epoch, privilege and selected paging/interrupt/direction controls are checked.
Every covered page follows owned level-specific translation storage, user/write
permissions and the corresponding image, stack or heap owner. Numeric saved
records do not construct authority in the kernel.

The [native run](native/run.json) passes all four original metadata iterations
and enclosing process retirement in about 349 seconds. The extra observations
retain **104 boundary snapshots for 52 actual helper calls**: **44 stat outputs**
and **eight positional-write inputs**, covering **96 translated page instances**.
All retained owner, table, entry and control data agree exactly across each
helper. Transient arithmetic flags may differ; both boundaries independently
require supervisor execution with interrupts and direction flag clear.

[Saved readback](native-readback.json) joins every helper record to the actual
ordinary syscall sequence, verifies snapshot digests, rechecks all physical
owners/translations, derives kernel memory spans from ELF program headers and
checks actual linked context/table/registry symbols. The four role bindings
match both the compiler's owner record and target IR. It executes no guest code.
The companion [metadata readback](metadata-readback.json) independently replays
all 124 filesystem/table transitions, 44 complete stat outputs and 44 observed
clock returns of the original 136-syscall cohort.

All **149 model/reader tests**, including eight owner test methods with explicit
security counterexamples, pass. [Twelve offline saved-record controls](controls/controls.json)
are rejected. Semantic controls coherently rebind snapshot digests, so stale
cells, missing pins, physical aliases, foreign tables/leaves, stale entry and
disabled write protection cannot pass merely by updating a hash. Additional
controls reject missing/reordered calls, false page evidence, wrong direction
and wrong model identity. Only saved JSON records are altered; no ELF, debugger,
register or guest-memory mutations are executed.

The [comparison](artifact-comparison.json) preserves all seventeen core
objects/images, thirteen audits and four other metadata artifacts exactly.
Kernel SHA-256 remains
`47b785aca3794d171d508a3fdbb616057a613016c1783ca9ec95236dbba2199f`.
Current first-party source/fixture/boot-resource identities are also frozen.
Diagnostic filename and baseline provenance differences retain separate actual
versions, without claiming retained intermediate text equivalence. There is no
new GNU decoder run. The existing 12 fresh / 13 joined cleanup-block and 76 fresh
/ 83 joined helper-site reach counts retain their prior limited meanings.

The four changed runtime/test source versions remain identical before and after
the command. Its peak cgroup memory is 668815360 bytes, swap is disabled and every
recorded memory-event counter is zero. The existing finite 600-second metadata
deadline, observations, VM boundaries and host protections remain intact.
[The invocation](native/invocation.json) records the optional owner-test profile.
Default test selection, executable kernel behavior, core language meaning and
canonical production selection are unchanged. No safeguard notice occurred.

The [source record](source-identity.json) binds **175 selected inputs**, four
local and 171 inherited, and the [artifact record](artifact-identity.json) binds
89 current/historical artifacts. [Frozen replay](replay.json) restores these
selected inputs, reproduces thirteen reports, all models and the exact saved
owner/metadata readbacks. It runs no ELF, compiler, GNU decoder, GDB or VM:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-owner-boundaries/replay.py linux-kernel/.cache/x86_64/owner-boundary-replay-new
```

Use a new output directory. [Archive validation](validate-archive.py) checks the
selected bindings and hashes. Review remains single-agent with separate source,
ELF, model and saved-record interpretations; no independent human security
review is claimed. The underlying memory/lifetime/evidence risk remains high.
The first archive replay stopped while restoring an inherited record that lacks
the optional packed digest. Its [failure and exact reader](history/initial-replay-failure.json)
remain recorded. The final replay follows the preceding reader's optional-field
handling while always verifying source bytes; the manifest binds every packed
reference separately. No guest code ran in that failed archive replay.

Boundary equality does **not** establish a continuous stable mapping/ownership
lifetime, actual private-counterpart loans, every helper/caller, complete
entry/callee/return/fatal/exception or whole-syscall effects. Other lengths,
refusals and fallback paths, canonical clean production reconstruction and full
Linux distribution/container/VM-host acceptance remain required. Stage 3,
initial Step 4 and the full original kernel objective remain unfinished.
