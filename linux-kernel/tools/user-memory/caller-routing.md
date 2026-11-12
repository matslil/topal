# Owned caller routing notes

This is a source-review record for `TOPAL-SYSTEMS-USER-EXTENT-001`, not a
complete caller inventory or proof that every direct user access is qualified.
The [signal checkpoint](results/stage-3-source-signals/README.md) records native
observations separately from the following control-flow reasoning.

| Previously named area | Current owned selection | Evidence boundary |
| --- | --- | --- |
| Initial process-name and stack-limit output | Both adapters choose complete owned output admission. Their calls to the old `emit_stack_output_extent` helper occur only in the predecessor `else` branches. | [Adapter source](../../../src/topal-compiler/src/systems_process_entry_completion.rs) and [startup native checkpoint](results/stage-3-source-startup/README.md). This does not qualify every initial-vector construction or publication obligation. |
| Signal reset after the final shell command | The final read initializes the process table before publishing command phase six. Initialization imports/consumes prelude signal state and publishes the active table. On subsequent syscall entry, active-table action/mask dispatch precedes the old prelude signal selector. | [Final-read source](../../../src/topal-compiler/src/systems_process_entry_read_exec.rs), [initialization](../../../src/topal-compiler/src/systems_file_descriptions.rs), [entry order](../../../src/topal-compiler/src/systems_process_entry_start.rs) and [signal dispatch](../../../src/topal-compiler/src/systems_process_signals.rs). The new cohort requires that actual table to be active and tests its service; it does not force the old fallback route. A complete reachability/producer inventory remains required. |
| Header readiness polling | The owned adapter admits both validated header forms, repeated polls and arbitrary input `revents`, retaining the open immutable description, complete copy checks and ordered read consumption. The external form retains its terminal probes; clean direct reads remain valid. | [Header-read adapter](../../../src/topal-compiler/src/systems_process_entry_read_exec.rs) and [new native poll/regression record](results/stage-3-owned-poll/README.md). Source-issued buffer cases now reach the common poll path, and unchanged pinned BusyBox separately exercises the external profile. General descriptor/event/count/timeout polling remains required. |

The process transfer staging extent is 256 bytes beginning at table offset
1408. The process umask begins at offset 1664, before the signal action array
at 1792. Signal preparation and return use the shared staging wipe; testing
that wipe must preserve the adjacent umask and all other process state. The
signal observer compares all 8192 table bytes against the operation's permitted
action/mask update, and independently requires the entire 256-byte staging
extent to be zero. It does not erase or ignore adjacent ownership.

The old fallback labels remain emitted. Source reasoning about the normal
dispatch order is not native execution of those labels, a claim that arbitrary
corrupt private state is unreachable, or permission to delete their checks.
Every remaining ownership, paging-mode, stable-observation, direct/shared-caller
and production reconstruction obligation remains in force. The source identities
used for these notes are retained with the selected signal and readiness evidence.

The [selected emitted-call census](results/stage-3-call-census/README.md) now
checks all 91 calls to the two owned helpers in the selected provider against
ELF bytes and LLVM disassembly. The enclosing ordinary process/poll cohort
reaches 77 sites and leaves 14 unobserved: three bounded fallback inputs,
positional and serial descriptor reads, named-directory publication, four
process-pipe calls and four volatile-file calls. Exact offsets, immediate
emitter locations and preceding native labels are retained. These are first-hit
observations before invocation, not admission/return qualification or a complete
direct-access inventory. They identify the next active-descriptor source cases
and fallback classification needed without weakening any acceptance gate.

The subsequent [ordinary FIFO cohort](results/stage-3-source-pipes/README.md)
reaches all four previously unobserved process-owned pipe sites. Its 144 source
calls verify complete selected table/FIFO transitions, buffer refusals, actual
byte transport and exact endpoint/ring retirement. The two matching-kernel
cohorts reach 81 of 91 sites in union; ten remain unobserved in that union.
This does not replace the complete direct-access and caller qualification gates.

The [ordinary storage cohort](results/stage-3-source-storage/README.md) adds the
four volatile-file helpers, immutable-file pread and named-directory publication.
Its 208 source-issued calls compare selected table/filesystem transitions,
failure atomicity, output, independent positions and complete anonymous-file
reclamation. Exact matching-kernel/census checks establish an 87-of-91 union
with the preceding poll and pipe cohorts. Serial descriptor read and three
bounded fallback inputs remain unobserved in that union. The complete direct-
access, length/error/lifetime and production gates remain separate and required.

The [ordinary serial cohort](results/stage-3-source-serial/README.md) reaches the
remaining serial descriptor site through real stdin reads. Its 64 source calls
observe actual UART data consumption, refusal before consumption, copy windows,
table preservation and bounded 512-to-256 admission. Eight host messages travel
through the existing VM serial connection after ordinary application markers.
The four matching-kernel cohorts now reach 88 of 91 sites in union. Three bounded
fallback inputs remain unobserved; their dispatch/ownership classification and
the complete direct-access/caller obligations still require explicit evidence.

The [selected result-gate audit](results/stage-3-transfer-guards/README.md)
checks immediate carry refusal after return-word erasure at all 91 calls and
the exact local bounds of the three remaining fallback read slices:

| Fallback | Selected emitted call offset / admitted bytes | Source lifecycle and local read boundary |
| --- | --- | --- |
| Named-root condition output | 88236 / 14 | [Output emitter](../../../src/topal-compiler/src/systems_process_entry_output.rs) requires context byte 418 to equal four, plus the preceding command/condition state. Its one-byte loop reads indices 0–13 before private content checks and serial publication. |
| Exec signal reset | 94708 / 32 | [Completion emitter](../../../src/topal-compiler/src/systems_process_entry_completion.rs) selects the reset with context byte 418 equal to six. It reads eight-byte fields at offsets 0, 8, 16 and 24 after input admission. The earlier active-table routing reasoning remains separate from native execution or full reachability proof. |
| Missing root pathname | 101802 / 9 | [Completion emitter](../../../src/topal-compiler/src/systems_process_entry_completion.rs) requires context byte 418 equal to four and pathname-query state. It reads eight bytes and the terminator at offset eight. An absent lookup has no metadata output admission or transfer. |

The two phase-four paths precede the final command read's phase-six/table
activation. Active-table interception therefore does not justify dismissing
all three fallback paths as unreachable. The fresh unchanged BusyBox exit
cohort reaches 38 sites already in the preceding union; the union remains 88
of 91. No historical check is removed. The audit's direct-entry checks concern
decoded branches into these local slices only; embedded data, unknown decoding,
indirect/exception entry, complete access inventory and stable observation
still need their own evidence.

The [linked process graph](results/stage-3-process-accesses/README.md) now lists
31016 instructions and their potential addressed/implicit-stack operations.
Exact LLVM byte checks and GNU re-decoding agree on every instruction boundary
and all 36938 direct-control edges. Two real branch destinations require targeted
decoding after the linear listing loses alignment. All 88 preceding native
first-reach sites occur in this graph. Exec signal reset and missing-path calls
are in its 90-call set; the named-condition call is outside it. The 91-call
emission census and all three fallback obligations remain intact. The graph's
listed roots, call-return assumptions and 18 external/six indirect boundaries
still require complete entry/callee contracts. Its 14584 potential-access
instructions are a review inventory, not user-pointer classification or proof
of admission dominance, transfer direction or stable mapping lifetime.

The [callback binding and ordinary-execution record](results/stage-3-callback-bindings/README.md)
now follows all five source-controller entries from actual bootstrap relocations
and argument pushes through private context registration to the six indirect
calls. All 357 allocated sections match their source objects after 978 relocation
applications. The final ordinary storage cohort observes all six sites and 1168
returns, checks the actual registered targets, supervisor execution and ABI/
captured-argument/registry/full-process-table preservation, and compares all
38541 live section bytes on first use of each controller. The separate unchanged
BusyBox exit observes 77 dispatch calls. Hardware supervisor write permission
remains present; this is not hardware read-only code qualification. Recorded
source/runtime call certificates, complete callee effects, alternate entries,
pointer authority, admission dominance and stable lifetime retain their own
obligations. No metadata association grants new kernel authority or closes a
fallback gate.

The [emission/source join](results/stage-3-process-origins/README.md) now covers
all 231663 selected function bytes and all 14584 potential-access instructions
in the listed graph. Its 45005 spans/9380 locations identify 50 actual writer
files after exact whole-function relocation reconstruction. The
[writer index](results/stage-3-process-origins/writer-index.md) makes source review
tractable: for example, 1006 completion instructions normalize private leaf-table
status rather than copy application data. Append sites and nearby labels remain
diagnostic context; ownership/lifetime, true pointer origins and admission
dominance still need evidence for each semantic class. Neither a register name
nor source placement is a safety classification. Existing root/callee/fallback
and canonical-production obligations are unchanged.

The [conditional scalar check](results/stage-3-fixed-transfers/README.md) now
binds 27 actual scalar accesses in nine local transfer regions to their extent
success edges, candidate aliases, direction and complete byte bounds. It
recomputes the listed graph's direct edges/actual roots, requires nonvacuous
reachability and rejects alternate local entries. Separate GNU decoding and
backward predecessor review agree on all selected operands. This includes
local exec-reset/missing-path reasoning without claiming those fallbacks were
natively exercised. The named-condition fallback remains outside the graph.
The shared helper's soundness/preservation, stable owner lifetime, complete
entry/callee/exception contracts, all other semantic access regions and full
syscall effect/error ordering remain separate requirements. The fresh pinned
exit preserves every compared native artifact and does not close those gates.

The [REP checkpoint](results/stage-3-copy-transfers/README.md) adds raw
candidate/count/DF dataflow for fifteen admission paths to fourteen copy
instructions. Clock time and resolution share a copy, so their success edges
are checked as one complete admission group; the intervening sampler's actual
register effects preserve the buffer/staging registers. Ordinary storage now
completes all fifteen selected paths with 166 copies and 118 refusals; poll and
unchanged pinned exit provide separate matching-kernel observations. The
observer checks actual helper preservation, complete byte transport and
translation/control observations at copy boundaries. It exposes counterpart
expressions without granting ownership; neither boundary equality nor distinct
expressions proves continuous lifetime or nonaliasing in general. Other access
regions, complete owner/helper/entry/callee/exception contracts, all fallback
and production obligations remain required. Canonical selection is unchanged.

The [finite-loop/unrolled checkpoint](results/stage-3-staged-transfers/README.md)
adds sixteen staged regions and 208 user-access instructions. Actual preparation
paths preserve or correctly reload the candidate; finite indices and every
qword offset remain within the exact admission. Storage completes all sixteen
regions and pinned exit fourteen, with exact payload and boundary mapping/
control checks. These regions observe no refused admission in those runs, so
their native invalid-buffer/refusal coverage is still open. The
[joined caller/access index](results/stage-3-staged-transfers/caller-access-index.json)
lists forty helper sites with selected local intervals and 249 distinct
user-access instructions, plus 51 sites without such intervals. It does not
qualify whole callers or every remaining access. GS/captured-entry/counterpart
owners, complete helper/root/callee/exception contracts, continuous lifetime,
all fallback obligations and canonical production acceptance remain required.

The [ordinary staged-refusal cohort](results/stage-3-staged-refusals/README.md)
adds 91 refused admissions and twenty completed transfers across those same
sixteen regions. Actual source-adapter syscall boundaries arm the selected
observers and associate every refusal with its pointer, `EFAULT` and unchanged
phase. The selected counterpart bytes remain unchanged across refusal; complete
source-fixture syscall/state assertions still run. This closes the ordinary
invalid-buffer observation gap for those regions, not their complete owner,
lifetime or whole-syscall obligations. The new cohort observes first reach at
these sixteen sites only; it does not repeat the broad census/controller/REP
observations or change their historical scope. Default observers in the fresh
pinned BusyBox cohort preserve exactly the preceding native observation object.
Kernel/application/boot/output bytes remain unchanged, as do the forty local
intervals/249-instruction index and the 51 sites without selected intervals.

The [additional scalar record](results/stage-3-additional-scalars/README.md)
adds 33 selected intervals and 51 user-access instructions. It proves exact
per-interval candidate/count setup, preserved aliases (including shifted poll
output), checked header preparation, raw byte/word bounds and direct-graph
success-edge dominance. The joined index now has 73 helper sites/300 selected
user instructions and eighteen sites without intervals. Repeated scalar probes
qualify only their own invocation; the later full poll readback, private loop
staging and whole-caller effects remain separate. The wait-status preparation's
additional read is explicitly unqualified. Fresh pinned exit preserves all
preceding native observations and reaches fourteen selected calls, without
claiming new native scalar-transfer coverage. Complete owners/lifetime,
helper/root/callee/exception, all fallback and production obligations remain.

The [staging-copy checkpoint](results/stage-3-staging-copies/README.md) retains
the admitted candidate/count through selected console and descriptor preparation
and adds six exact-byte staging paths, including cwd and sparse read/pread.
The changed kernel's fresh index contains 79/91 helper sites and 306 selected
user-access instructions, with twelve sites lacking local intervals. Explicit
caps, closed preparation/lookup effects, DF, incoming edges and success cuts
remain conditional on complete owners/lifetime and helper/entry/callee contracts.
Serial pointer emission is improved, but its partial-count interval remains open.
The new kernel has its own actual native coverage union; older observations are
not relabeled. Whole-caller, fallback and canonical-production obligations remain.

The [fixed-pipe audit](results/stage-3-fixed-pipe-audit/README.md) checks the
pair output/readback and readable-count output through exact preparation and
normal-return lookup contracts. These two conditional regions add three access
checks in a separate report. GNU readback, twelve offline artifact controls,
eight new models, exact frozen replay and the ordinary pipe VM regression pass.
No executable kernel bytes change. Fresh per-access observation, actual private/
output/stack owners, nonaliasing, continuous lifetime and complete entry/callee/
exception effects remain required. The published 79-site/306-instruction index
and all twelve sites without published intervals remain unchanged.

The [native fixed-pipe checkpoint](results/stage-3-fixed-pipe-native/README.md)
then observes 24 completed fixed outputs, 40 refused admissions and 28 scalar
accesses through 144 ordinary source syscalls. Separate saved-record readback
checks source order, queue-derived count payloads, pair values and translation/
exclusion observations. Kernel/application/boot bytes remain unchanged. The
recomputed six-report conditional index has 81 helper sites and 309 user-access
instructions, leaving ten sites without intervals. Complete owners/nonaliasing,
continuous lifetime, helper/entry/callee/exception/fallback and canonical
production obligations remain open; neither boundary equality nor observed
disjoint ranges grants that authority.

The [stat output checkpoint](results/stage-3-stat-outputs/README.md) adds two
conditional admission paths to one shared 144-byte copy. Exact setup, wipe,
restricted preparation effects, six normal-return callee contracts and saved
register retirement preserve candidate/count/DF. GNU readback, fourteen offline
artifact controls, 82 model/reader tests and exact selected replay pass.
Ordinary source execution observes sixty completed copies and twenty output
refusals among 88 stat queries; eight lookup errors precede registered output
events. Separate record correspondence checks selected independent constants
and relations while retaining the runtime private oracle for dynamic fields.
The joined seven-report index now has 83/91 sites and 310 distinct user
instructions, with eight sites still without intervals. Kernel bytes are
unchanged. Actual owners/nonaliasing, continuous lifetime, complete entry/helper/
callee/return/fatal/exception guarantees, all three fallbacks and canonical
production qualification remain required.

The [serial prefix checkpoint](results/stage-3-serial-prefix/README.md) checks
positive capped admission and bounded UART accumulation before copying only
the received prefix. Ordinary exact-copy contracts remain exact. Four source
executions observe twelve copies, sixteen refusals and four short 8-of-256-byte
outputs; the unchanged pinned regression also passes. GNU operands/reverse cuts,
twelve offline controls, 88 model/reader tests and exact frozen replay pass.
Kernel bytes are unchanged. The joined conditional index reaches 84/91 sites
and 311 user instructions, leaving seven sites without intervals. Actual owners/
nonaliasing, continuous lifetime, full helper/entry/callee/return/fatal/exception,
all three fallback and canonical production obligations remain required.
