# Step 4: owned user-memory extents

`TK-DEC-283`, `TOPAL-REQ-SYSTEMS-USER-EXTENT-001`,
`TOPAL-SYSTEMS-USER-EXTENT-001`, `TOPAL-COMP-SYSTEMS-USER-EXTENT-001`.

The [design](design/owned-user-extents.md) begins the next prerequisite for
general virtual memory. The production kernel still has the preceding finite
MM geometry and legacy user-buffer validators. This increment does not yet
change that executable behavior or complete initial Step 4.

The [selected helper-call census](tools/user-memory/results/stage-3-call-census/README.md)
checks 91 emitted calls against actual ELF/disassembly and observes first reach
of 77 in ordinary process execution. Fourteen sites remain unobserved in that
cohort. The [subsequent source FIFO cases](tools/user-memory/results/stage-3-source-pipes/README.md)
add four newly reached sites, for an 81-of-91 union, and verify 132 selected FIFO
state transitions in 144 source-issued syscalls. All 17 selected kernel objects/images
match the prior checkpoint. The [subsequent ordinary storage cohort](tools/user-memory/results/stage-3-source-storage/README.md)
adds six sites and 208 source-issued syscall checks, bringing the matching-kernel
union to 87 of 91. The [ordinary serial cohort](tools/user-memory/results/stage-3-source-serial/README.md)
adds real UART-consumption and copy observations in 64 source-issued syscalls,
bringing the union to 88 of 91. Three bounded fallback sites remain unobserved. The
census changes test metadata and observation only. Complete direct user-access
inventory, admission/error/lifetime qualification and production reconstruction
remain acceptance blockers. First reach alone discharges none of those gates.

The [subsequent result-gate audit](tools/user-memory/results/stage-3-transfer-guards/README.md)
checks all 91 immediate carry-refusal branches and the exact admitted bounds of
seven static read instructions in the three fallback slices. A fresh unchanged
BusyBox exit passes signal ownership/terminal retirement checks and reaches 38
already observed sites; the union remains 88 of 91. All 17 compared kernel
objects/images and the external console transcript are unchanged. Five offline
first-party artifact controls and six post-check commands pass. The 62-input
record and exact archived static-audit replay retain this limited evidence.
The phase-four fallbacks cannot be dismissed using the phase-six table dispatch
argument. Native fallback reach, the complete direct/shared-access inventory,
global dominance and lifetime/mode/ownership qualification remain required;
no production activation or full Stage-3 completion is claimed.

The [linked process access inventory](tools/user-memory/results/stage-3-process-accesses/README.md)
adds a 31016-instruction direct-control graph with exact linked bytes and
independent GNU instruction-length/edge/closure checks. It recovers two real
branch destinations omitted by the linear listing and records 14584 potential
addressed/implicit-stack operations for semantic review. All 88 preceding native
first-reach sites belong to its 90-call set; one of 91 emitted calls is outside
this graph. That distinction does not close any fallback obligation. A fresh
unchanged BusyBox exit, native graph binding, six post-checks and exact archived
LLVM/GNU replay pass; all 17 compared kernel objects/images, the pinned application,
boot resource and console transcript are unchanged. The 70-input selected record
does not establish a complete producer, all-kernel graph, pointer provenance,
admission dominance, stable access lifetime or production qualification.

The [source-controller boundary checkpoint](tools/user-memory/results/stage-3-callback-bindings/README.md)
binds the six indirect-call sites to five actual root-supplied source entries,
389 functions and every relocated byte of 357 allocated object sections. Its
final ordinary storage cohort observes all six sites and 1168 ABI/private-state
preserving returns alongside 208 source-issued syscalls. All five controllers'
live sections match the linked image; a separate unchanged BusyBox exit observes
77 dispatch returns. Supervisor execution is verified while supervisor write
permission remains present, distinct from hardware read-only text. Nine offline
binding counterexamples, six post-checks and exact archived binding replay pass.
All 17 compared kernel artifacts and the application/boot/console baselines are
unchanged. The 93-input selected record does not replace complete callee-effect,
entry/ownership/lifetime, direct/shared-access or production qualification.

The [process emission/source index](tools/user-memory/results/stage-3-process-origins/README.md)
adds test-only caller-location instrumentation and joins all 14584 listed
potential-access instructions to 50 emitter files. Its 45005 spans cover all
231663 function bytes; complete provider-to-linked reconstruction checks all
19 relocations. Independent span expansion, five malformed-metadata controls,
fresh unchanged BusyBox execution, six post-checks and exact archived replay
pass. All 17 compared kernel artifacts, the pinned application, boot resource,
console transcript and original 91-call census are unchanged. The 114-input
selected record supports source review, not a new claim of pointer authority,
admission dominance, stable lifetime or complete production qualification.

The [conditional scalar admission checkpoint](tools/user-memory/results/stage-3-fixed-transfers/README.md)
connects 27 actual read/write instructions in nine local transfer regions to
their successful extent checks, exact candidate aliases, direction and byte
bounds. It verifies the listed direct graph's edges/roots before nonvacuous
success-edge dominance, with separate GNU decoding and backward predecessor
checks. Five malformed-metadata controls and five algorithm/operand controls
pass. A fresh unchanged pinned exit, all 22 model/reader tests and six other
post-checks pass; all 17 compared native artifacts and console bytes remain
identical. The 116 selected inputs explicitly inherit 113 unchanged preceding
snapshots; exact Python-only source-join/scalar replay passes. This evidence is
conditional on helper, entry/callee/exception and stable owned-mapping contracts;
it does not establish native execution of every selected transfer, whole-syscall
effects, other access regions or production qualification. The three fallback
obligations and complete Stage-3/Step-4 acceptance remain open.

The [REP admission and execution checkpoint](tools/user-memory/results/stage-3-copy-transfers/README.md)
checks fifteen candidate/count paths to fourteen repeated copies, including
the shared clock output's two admission edges and intervening sampler effects.
Fresh ordinary storage completes all fifteen paths (166 copies, 118 refusals);
ordinary process/poll completes fourteen (170 copies, 118 refusals), and unchanged
pinned BusyBox completes five command copies. Native observations check helper
register preservation, exact copied bytes, pointer/count/DF and translation/
control state at copy boundaries. Separate GNU/backward checks, six offline
controls, all 28 model/reader tests and six other post-checks pass. All seventeen
compared native artifacts and console bytes remain unchanged. Two observer
failures, an explicitly stopped diagnostic run and a readback path-selection
failure retain their original statuses. The 119 selected inputs and exact
archived replay do not qualify a complete producer, continuous lifetime,
counterpart/table/backing ownership, every other access class or production
reconstruction. Canonical activation and complete Stage-3/Step-4 acceptance
remain blocked on those existing obligations.

The [finite-loop/unrolled staged checkpoint](tools/user-memory/results/stage-3-staged-transfers/README.md)
adds exact candidate/preparation/bounds checks for sixteen regions and 208
user-access instructions, with separate GNU decoding and backward dominance.
Fresh storage completes all sixteen (twenty copies); unchanged pinned exit
completes fourteen (eighteen copies). Both preserve exact payloads, captured
arguments, GS binding and boundary translation/control observations. No refused
admission occurs at these new regions in those cohorts; native invalid-buffer
coverage remains required. Six malformed first-party artifact controls, all
34 model/reader tests and six other post-checks pass. All seventeen compared
native artifacts, application/boot/console bytes and preceding metadata remain
unchanged. The 122 selected inputs replay exactly. The joined index identifies
forty helper sites with local intervals and 249 distinct user accesses, plus
51 unreviewed sites; these are not whole-caller safety proofs. Complete owner/
lifetime/helper/entry/callee/exception, fallback and production obligations
remain open, and canonical production selection is unchanged.

The [ordinary staged-refusal checkpoint](tools/user-memory/results/stage-3-staged-refusals/README.md)
now observes all sixteen selected regions with 91 refused admissions and twenty
completed transfers in the existing 796-syscall source fixture. Each refusal
is associated with the source syscall's actual pointer, `EFAULT` result and
unchanged phase; selected counterpart bytes, helper registers and captured
arguments remain unchanged across admission. Scoped breakpoints retain the
complete selected check/copy interval while avoiding unrelated heap work.
Unchanged pinned BusyBox separately preserves its complete preceding native
observations. All seventeen kernel artifacts, application/boot/console bytes
and original access metadata remain unchanged. Thirty-nine model/observer
tests, two fixture build tests, interface readback, six offline diagnostic
controls and all seven post-checks pass. Two deadline failures and a pre-boot
pinned-image configuration failure remain explicitly failed. The specific
ordinary-buffer refusal gap is covered; complete owners/lifetime, other access
regions, all fallback and canonical production obligations remain open.

The [additional scalar checkpoint](tools/user-memory/results/stage-3-additional-scalars/README.md)
checks 33 more local regions and 51 user-access instructions, including shifted
poll output, header-byte preparation/readback and per-invocation byte/vector
probes. Separate GNU decoding agrees on 279 instructions and all 51 backward
success-edge cuts; eight offline first-party artifact controls are rejected.
The joined index now has 73 selected helper sites and 300 user instructions,
with eighteen sites lacking intervals. It does not qualify whole callers or
hide the wait-status preparation's separate ownership/access obligation.
Fresh pinned BusyBox preserves its entire preceding native observation object,
all seventeen compared kernel artifacts and application/boot/console bytes.
The three preceding access reports remain byte-identical. Forty-five tests,
both fixture builds, all seven post-checks and exact selected-source replay
pass. New native scalar-transfer observation is not claimed; helper/entry/
callee/exception, complete owner/lifetime, remaining access, fallback and
production obligations remain required. Canonical production is unchanged.

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

## Stage 3 source-built shell and command transfers

The [shell checkpoint](tools/user-memory/results/stage-3-source-shell/README.md)
adds 38 source callbacks for signal actions, scalar shell output, uname/cwd,
root pathname input, terminal misses and header bytes. Their 299 syscalls pass.
Five further source callbacks collect short command reads and issue 65 syscalls,
including zero counts and refused retries. The combined VM observes 796 syscalls
across four images, retaining all preceding startup, file and runtime cases.

The first signal installs use actual read-only source data. Successful cwd and
header output ends immediately before an unmapped page, and other buffers cross
pages beyond the predecessor heap. Refused transfers preserve protected state;
the source program preserves input, unchanged output tails, credentials and
unrelated heap bytes. Callbacks return through the normal generated ABI adapter,
whose original registers and stack remain observed.

Independent readback reconstructs all five commands and the complete 512-byte
private state after receive, refusal, short delivery and completion. It proves
that received serial bytes survive failed output admission, unread suffixes
remain available, and consumed bytes are physically zero. Collection in C is
explicit: the final syscall returns the remaining suffix length; the source
callback returns the aggregate length to the enclosing witness.

All 7200 linked code bytes, actual read-only signal-action bytes, both syscall
adapters and 68 generated call sites are separately parsed. The complete user
stack cost remains 1720 of 2048 bytes by taking the largest mutually exclusive
delegated C call path. Seventeen object/image comparisons preserve the preceding
kernel and boot image; all 7606 serial bytes remain identical. Two build-only
tests and six post-check commands pass. Native peak memory is 353239040 bytes
with no limit/OOM/kill events or swap; host boundaries and resource policy remain.

Review remains single-agent self-review with independent byte/state models and
runtime observations, not independent human review. Header-poll negative cases
are not reached by this clean-header witness; specialized signal/reset and
vector-output cases and complete caller/native ownership/mode/stability
qualification remain required. Canonical production binding, exact production
qualification and cold reconstruction remain pending. Stage 3, initial Step 4
and the full original kernel goal remain unfinished.

## Stage 3 source-built exec vector transfers

The [exec checkpoint](tools/user-memory/results/stage-3-source-exec/README.md)
adds a separate source-built cohort for pathname, argv and environment admission.
Four invocations pass 116 ordinary exec refusals and twelve heap calls. These
requests cover unaligned page/directory crossings beyond page 34, readable
read-only vectors, nested string faults, inaccessible terminators and argument/
environment count and aggregate-byte limits. The enclosing application completes
its existing successful replacements and final retirement.

Every refusal preserves the complete process table, signal/thread registrations,
shell/file controls and private command bytes, and clears all 1536 private exec
staging bytes. Native observations also check callee registers, heap restoration
and terminal retirement. Separate ELF/request readback verifies the source code,
read-only inputs, normal call and conservative 1304-of-2048-byte source stack
bound. It does not claim a transcript of every nested user-memory load.

Seventeen comparisons preserve the preceding kernel and boot-image bytes; all
7606 serial bytes match. A fresh build preserves the preceding combined-transfer
fixture image exactly, and both build-only tests and six post-check commands
pass. The bounded native command peaks at 1131925504 bytes with no limit/OOM/kill
events or swap. A readback byte-count correction is retained separately from
the passing native run. Review remains single-agent self-review with separate
artifact parsing and observations, not independent human review.

The agent guidance now explicitly says that repository defects, resource failures
and successful retries cannot establish the reason for an AI safeguard action.
The notice and proposed operation must be retained when available, and inferred
explanations labeled. No safeguard denial occurred in this cohort.

Header-poll needs the sectioned profile's actual startup/lifecycle, not just its
ELF tuple. The pinned external digest remains a separate compatibility gate.
Specialized signal/reset and vector-output cases, complete caller and native
ownership/mode/stability qualification, canonical production binding and cold
reconstruction remain required. Stage 3, initial Step 4 and the full original
kernel objective remain unfinished.

## Stage 3 source-built signal transfers

The [signal checkpoint](tools/user-memory/results/stage-3-source-signals/README.md)
adds four source invocations with 92 action calls, 92 mask calls and twelve
heap calls. Ordinary application arguments exercise complete page/directory
crossings beyond page 34, code-resident input, refused complete copies, error
order, normalization, reset/restore, block/unblock/set and overlapping buffers.
Eight valid updates remain committed after old-value output faults, preserving
the explicit signal contract rather than imposing blanket error rollback.

The native observer compares the entire 8192-byte process table with exactly
the allowed update and requires all 256 transfer-staging bytes to be zero.
It preserves the adjacent umask and other state. The source restores its original
action/mask and heap contents; callee registers and terminal retirement pass.
Separate set-based readback checks every recorded action/mask transition and
old-value result, all 36 read-only inputs, eight successful overlaps and eight
output-fault commits. It verifies all 10144 linked text bytes and a conservative
1496-of-2048-byte source-call stack bound. Complete-table comparisons are native
observer evidence, not reconstruction from an archived full-table transcript.

Seventeen comparisons preserve the preceding kernel and boot-image bytes;
all 7606 serial bytes match, and a fresh build preserves the preceding transfer
fixture image exactly. Two build-only tests and all six post-check commands
pass. Native peak use is 359383040 bytes with zero limit/OOM/kill events and
validation swap. The initial observer staging-bound failure remains failed
and separately retained; a post-run overlap-classification correction also
remains recorded. Neither is an AI safeguard denial or kernel change.

[Routing notes](tools/user-memory/caller-routing.md) distinguish the active
process-owned signal path from old prelude fallbacks and identify predecessor-
only stack-output checks. They do not claim a complete reachability or user-
access inventory. Review remains single-agent self-review with separate native
observations, artifact parsing and a set-based oracle, not independent human
review. Header-poll coverage, complete caller and native ownership/mode/stability
qualification, canonical production binding and cold reconstruction remain
required. Stage 3, initial Step 4 and the full original kernel goal are unfinished.

## Stage 3 owned header readiness

The [owned poll checkpoint](tools/user-memory/results/stage-3-owned-poll/README.md)
extends the experimental owned selection to bounded polling from either
validated header form. It ignores incoming `revents`, allows repeated polling
without advancing position, and consumes readiness on successful byte delivery.
Clean direct reads remain valid. External terminal probes, descriptor rights,
complete eight-byte input/two-byte result admission and all independent bounds
remain. This systems-only refinement uses the standing approval and changes
no core language rule or predecessor selection.

The final focused VM passes 32 polls, twenty header reads and twelve heap calls
in four source callbacks, alongside 56 shared-runtime calls. It covers read-only
result refusal, partial inputs, directory/page crossings beyond page 34,
result fields crossing pages, repeated polls and refused reads with readiness
preserved. Independent readback reconstructs record and private-state changes,
verifies all 8520 linked text bytes and four generated calls, and checks a
1648-of-2048-byte source stack bound. Fourteen component/predecessor objects
remain identical to the preceding checkpoint. Only the provider's process-entry
function changes among 163 native function bodies.

The existing 796-call source regression passes with complete stat/directory/
command/queue readback. Unchanged pinned BusyBox separately passes the external
profile, exact output and first-image retirement. A retained included-file
formatting failure required wrapping one function signature. The fresh final
poll VM uses the formatted producer; its seventeen kernel objects/images match
all three pre-format native builds exactly. Those earlier regressions certify
the matching executable bytes, not execution from the formatted source input.
Both build-only tests and all six final post-checks pass. A fresh build preserves
the preceding transfer-fixture image exactly, and serial output is unchanged.

Review remains single-agent authority/full-diff self-review with separate native
observations and byte/state reconstruction, not independent human review or
complete production qualification. Host/VM protections and unchanged deadlines
remain; all recorded memory-limit/OOM/kill counts and validation swap are zero.
The targeted poll-buffer gap is covered. Complete caller/user-access inventory,
native ownership/mode/stability evidence, canonical production binding and cold
reconstruction remain required. General polling and the full original kernel
objective remain unfinished; Stage 3 and initial Step 4 are still incomplete.
