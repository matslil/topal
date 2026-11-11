# Step 4: owned user-memory extents

`TK-DEC-283`, `TOPAL-REQ-SYSTEMS-USER-EXTENT-001`,
`TOPAL-SYSTEMS-USER-EXTENT-001`, `TOPAL-COMP-SYSTEMS-USER-EXTENT-001`.

The [design](design/owned-user-extents.md) begins the next prerequisite for
general virtual memory. The production kernel still has the preceding finite
MM geometry and legacy user-buffer validators. This increment does not yet
change that executable behavior or complete initial Step 4.

The first stage records complete range, translation-permission, actual table
and backing ownership, stable entry lifetime and no-touch empty-range laws.
It identifies shared and independent buffer validators requiring migration,
and separates historical startup witnesses from arbitrary syscall admission.
The [primary-source record](knowledge/x86_64/user-memory-research.json) verifies
four exact files in the pinned upstream archive without importing their code.

The [model and tests](tools/user-memory/README.md) exercise the new contract
independently of the existing emitter. Model observations cannot create native
ownership; hardware context/stability and complete syscall/caller/publication
proofs remain required. The four implementation stages are recorded in the
design and will be commits in the existing initial-Step-4 PR.

Risk is high because an incorrect permission, ownership or lifetime check can
admit access to kernel storage. This stage uses a single-agent authority and
full-diff review, a separate flat permission-lattice oracle and guarded model
read traces. It makes no independent reviewer or native security proof claim.


## Stage 1 qualification

The [49-input record](tools/user-memory/results/stage-1/qualification.json)
passes all five recorded commands: seventeen model tests, all four Rust coverage
tests, workspace formatting, the unchanged Rust file-size ceiling and whitespace.
The model cases include 8192 direction/path permission combinations and an
actual 400-page model extent. No production Rust, LLVM or Topal kernel source
changed in this stage.

[Independent readback](tools/user-memory/results/stage-1/independent-readback.json)
checks every snapshot and log, compares retained production source bytes to the
preceding commit, replays all seventeen tests using isolated archived model
bytes, and verifies 681 local Markdown targets. It is artifact/model evidence,
not an independent human review or native execution proof. Exact qualification
inputs remain frozen; report and traceability updates only record the outcome.

Peak use was 107311104 bytes, with zero limit/OOM/kill events and zero validation
swap under the unchanged wrapper and host reserve. An initial harness input
enumeration named a nonexistent frontend requirement file and stopped before
tests. Its [failure scope](tools/user-memory/results/stage-1-harness-path-failure/failure.json)
remains retained. Removing that path required no model or runtime correction.

Stage 1 is complete. At that checkpoint, native implementation, all-caller
migration and committed production qualification remained stages 2–4. The full
original kernel objective is still unfinished. See the
[replay notes](tools/user-memory/results/stage-1/README.md).


## Stage 2 native implementation and functional checkpoint

The native four-level walk now binds actual compiler-derived root/table/image/
stack owners and the current runtime heap through the owning gateway. The low
leaf has its own allocation; it is not a page within the five-page hierarchy.
Runtime heap identities are additionally checked against all static root
allocation slots. Empty ranges remain no-touch operations, and entry/mapping
owners remain held beyond validation through the caller's access lifetime.

The [2157-input checkpoint](tools/user-memory/results/stage-2/qualification.json)
passes eight commands, including three selected compiler tests, seventeen model
tests, locked workspace/all-target checking and doc tests. The ordinary full-root
VM passes 324 native extent checks and 80 real memory operations. The actual guest
initializes CR0.WP before user entry, preserving every other CR0 bit. The full
experimental call path uses 2272 of its 4096-byte reserve.

[Readback](tools/user-memory/results/stage-2/independent-readback.json) independently
checks snapshots/logs, root instruction/owner derivation, ELF/boot/resource bytes
and every native result. [Replay](tools/user-memory/results/stage-2/replay.json)
repeats the ordinary VM using archived first-party bytes and frozen observers.
Peak was 2203799552 bytes, with zero limit/OOM/kill events and swap.

The current instruction limits execution to ordinary development and functional
tests. Deliberate guest-state controls were not rerun after the three-source
ownership refinement. A failed earlier full-suite debugger assertion and a later
review-stopped run are retained with their original incomplete status. This is
not full workspace qualification or completed Stage 2; the existing production
profile is unchanged. The [checkpoint notes](tools/user-memory/results/stage-2/README.md)
explain exact scope, earlier evidence and remaining gates. All-caller migration
and production activation/qualification remain pending; initial Step 4 and the
original kernel objective remain unfinished.

## Stage 3 caller migration and ordinary-application checkpoint

The experimental owned selection now routes the frame-profile startup, stat,
pathname, file, shell and shared runtime user-buffer paths through current
owned extents. The private command cursor and serial queue preserve short or
refused transfers and are retired on exec/exit. These changes remain separate
from default production publication.

The ordinary first-image exit now follows the actual ownership transfer:
descriptor initialization has imported the prelude signal dispositions into
the process table and consumed the old slots. Terminal admission checks the
active table and absence of a child, requires the consumed signal slots to be
empty, then retires the current table and opaque thread registrations. It does
not require obsolete prelude installation witnesses to remain live.

The [selected application checkpoint](tools/user-memory/results/stage-3-application-checkpoint/README.md)
passes one fresh-kernel interoperability test using unchanged pinned BusyBox,
exact shell output and actual signal/command/table/heap retirement observations.
Six post-check commands pass, including locked workspace/all-target checking
and seventeen model tests. Fourteen object comparisons preserve the preceding
component objects and both predecessor objects exactly. The complete caller
stack bound remains 2248 bytes, with 4088 bytes of physical retirement layout
within the existing 4096-byte reserve.

Risk remains high around user-memory ownership, native caller liveness,
translation lifetime and retirement. Review is single-agent self-review against
the approved systems contract, supplemented by separate object comparisons and
runtime before/after observations; no independent human review is claimed.
The [agent guidance](../AGENTS.md#kernel-and-systems-validation) governs future
validation. Historical binary-patched fixtures were not rerun for this
checkpoint, and model checks do not stand in for missing native evidence.

Stage 3 remains incomplete. Source-built native boundary/error fixtures,
complete caller qualification, canonical production binding and committed cold
reconstruction remain required. The default production profile and the full
original kernel goal remain unchanged and unfinished.

## Stage 3 source-built boundary cases

The [source-built checkpoint](tools/user-memory/results/stage-3-source-fixture/README.md)
adds a compiled C component and backend syscall adapter to the construction of
a fresh first-party ABI witness. The new variant generates command assertions
over returned bytes only. The default constructor retains the exact output of
the committed generator; no completed application is patched.

Four invocations pass 56 observed user syscalls covering runtime output across
page/directory boundaries, refused transfers without partial writes, invalid
output candidates, console zero-count/error order and heap restoration. Actual
user stack/callee preservation and whole-process retirement pass. Independent
readback verifies code inclusion in the executable read-only load, the ABI
adapter and the conservative 528-byte source-call stack cost.

Seventeen exact object/image comparisons preserve the previous kernel and boot
image, and all 7606 serial bytes match the earlier clean ABI witness. Two
build-only tests and five static post-check commands pass. The initial prelude
padding timeout is retained as failed evidence. This remains single-agent
self-review with separate native observations and artifact comparisons, not an
independent human review or complete production reconstruction.

This source fixture covers shared runtime `uname`/console and memory paths.
Direct startup/stat/directory/command native boundary cases and the other
Stage 3/4 obligations remain required. The full original kernel objective is
still unfinished.

## Stage 3 source-built startup transfers

The [startup checkpoint](tools/user-memory/results/stage-3-source-startup/README.md)
extends the source-built fixture to stack-limit, initial random and process-name
output, plus optional-configuration pathname admission. The backend emits normal
calls during fixture construction; a linked adapter preserves the original
syscall arguments and ordinary callee registers. Its separate startup scratch
protects live credential vectors. The complete conservative user-call cost is
1832 bytes within the unchanged 2048-byte fixture bound, independently of the
kernel's existing caller reserve.

Four initial/replacement image sequences pass twenty source callbacks with 200
observed startup syscalls, in addition to the preceding 56 shared-runtime calls.
The observer checks the actual relocated credential group before/after each
call, refusal phase preservation, original heap contents, caller registers and
terminal retirement. Initial configuration lookup takes the sealed fallback;
after exec the active namespace owner takes the general lookup path without
advancing that fallback phase. Both routes reject invalid input and preserve
lookup-before-output order.

Separate ELF readback verifies all 2765 linked code bytes, both syscall adapters,
the startup bridge relocation and all five emitted calls with disjoint scratch.
Seventeen exact object/image comparisons preserve the preceding kernel and boot
image; all 7606 serial bytes remain identical. Two build-only tests and static
post-checks pass. The original stack-overlap failure and two observer failures
retain their original nonzero results and scope in the archive.

Review remains single-agent self-review with independent artifact parsing and
native observations, not independent human review. The default fixture's bytes
and production selection remain unchanged. Other direct caller boundary cases,
complete ownership/mode/stability qualification, canonical publication and cold
reconstruction remain required. Stage 3, initial Step 4 and the full original
kernel objective remain unfinished.

## Stage 3 source-built file transfers

The [file checkpoint](tools/user-memory/results/stage-3-source-files/README.md)
adds twenty source callbacks at actual bounded file operations. It exercises
four `fstat` records, five pathname-stat queries, six opens, four directory
reads and one profile-file EOF. Their 176 observed syscalls pass in addition
to the preceding 200 startup and 56 shared-runtime syscalls across four images.
All requests use the ordinary application ABI; generated calls are emitted
during construction of the fresh first-party witness.

The source program checks refused copies without partial output, page-split
pathnames, complete stat output across page/directory boundaries beyond the
predecessor heap, and EOF without a mapped destination. Directory results use
only 80 mapped bytes before an unmapped page despite a count of 512, checking
actual-record admission. Observations verify refusal preserves private staging
and lifecycle controls. Heap contents, credential vectors, original registers
and terminal retirement remain checked.

Separate readback reconstructs all native stat and directory bytes, including
padding, parses all 4685 linked code bytes and both adapters, and checks 25
generated calls with disjoint scratch. The runtime entry cannot call the
startup entry; the latter may delegate to file cases. Independent stack
accounting sums all frames in each possible path and takes their maximum,
including existing scratch/adapters/returns: 1720 of 2048 bytes. It does not
raise the fixture bound or alter the kernel caller reserve.

Seventeen exact comparisons preserve the preceding kernel and boot image;
all 7606 serial bytes remain unchanged. Two build-only tests and six post-check
commands pass. The earlier observer zero-length-read failure and C compile
failure are retained separately. Review is single-agent self-review with
independent parsing and native observations, not independent human review.

Remaining direct signal/path/output/header/command cases, complete native
ownership/mode/stability qualification, canonical production binding and cold
reconstruction remain required. The default fixture, production selection,
Stage 3's incomplete status and full original kernel objective remain unchanged.
