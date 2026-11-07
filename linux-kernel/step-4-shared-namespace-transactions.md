# Step 4: shared namespace transactions

`TK-DEC-281`; follows the
[systems design](design/shared-namespace-transactions.md) and
`TOPAL-SYSTEMS-NAMESPACE-{OWNERSHIP,TRANSACTION,ABI,ACTION}-001`.

## Stage 1: independent entries and inode ownership

The portable [filesystem owner](../src/topal-semantics/src/systems_volatile_files.rs)
now owns four independent directory-entry slots alongside four regular inodes
and eight resident pages. Entries contain exact raw component bytes and stable
inode identity. Object metadata, page ownership and descriptions stay separate;
link counts come from the actual entry associations. Existing open, stat, chmod,
unlink, directory reads, close and termination use that same table. The owner
rejects dangling identities, duplicate names/inodes, malformed components and
initial-link eligibility inconsistent with a live entry.

The [transaction model](../src/topal-semantics/src/systems_namespace_transactions.rs)
adds hard links by copied name or admitted live descriptor, rename replacement,
NOREPLACE and EXCHANGE. Source and victim inodes retain their identities,
contents, sizes, pages, permissions, atime/mtime and description state. Only
the specified links and ctimes/parent times change. Replacing a victim retains
it through other links or open descriptions; last-reference reclamation erases
its model page associations and bytes. Recreating a spelling issues a fresh
identity. Same-inode rename/exchange preserves the entire state and both names;
NOREPLACE still returns EEXIST.

Initial anonymous temporary files may link through a checked descriptor;
explicit unlinkable creation models O_TMPFILE|O_EXCL. A successful first link
consumes eligibility, so a later open-unlinked inode cannot be resurrected.
Readonly descriptor access is not a link-authority check for the prepared
owning credential and grants no write right. Names, inode numbers, same-label
pools and stale descriptors cannot confer authority.

Entry and inode capacity are independent. A missing-target rename reuses its
entry and can succeed with a full directory. Hard links may exhaust entries
while other inode slots remain free; anonymous files may exhaust inodes while
all entry slots are free. Capacity and other recoverable failures preserve
complete state, including allocation, metadata and identity counters.
Directory cookies now follow entry slots independently of object slots, while
retaining exact 255-byte raw names, 280-byte native record sizes, copy/cursor
ordering and the existing no-snapshot-across-mutation law.

The [primary research record](knowledge/x86_64/namespace-transactions-research.json)
binds six complete Linux 7.2.9 files. The actual archive hash matches the pinned
manifest and each cached source file matches its archive member byte for byte.
Research records all five syscall numbers, flags, same-inode/no-replace order,
temporary-link eligibility, metadata changes and whiteout's separate device
requirement. This is source research, not executed Linux adapter qualification.

The [independent tests](../src/topal-semantics/src/systems_namespace_transaction_tests.rs)
cover shared inode bytes with independent/readonly descriptions, all unlink/
recreate/retained-victim lifetimes, rename without capacity, full-state exact
deltas, exchange/link counts, same-inode no-ops, separate budgets, temporary-link
consumption, raw long names/cookies, foreign/stale/poisoned refusal, malformed
tables, terminal retirement and native unsigned flag adaptation. All **223
semantics tests pass**, including these fourteen new cases, with no failed or
ignored tests. Locked workspace all-target checking and all four requirement/
specification coverage tests pass. The systems ledger has **310 unique rules**;
formatting, the unchanged 2000-line Rust ceiling, local links and strict
whitespace checks pass. The normal bounded wrapper retained its adaptive
memory limit, host reserve and zero swap; no host safeguard changed.

Risk is high because association identity, replacement lifetime, complete
failure preservation, source grants and native argument order interact. Review
compares the actual owner consumers and exact state/observation oracles with
the approved systems design. Direct Rust model transitions and logical
retirement are not opaque Topal execution, physical erasure or syscall evidence.

## Stage 2: ordinary source components

The new [ordinary action](kernel/policy/namespace-action.t) calls
[namespace metadata helpers](kernel/support/namespace-metadata.t) and
[fixed byte helpers](kernel/support/namespace-bytes.t). It uses the existing
two-Nat/Boolean instance ABI for operation and seconds. The component caller
lends a real 4096-byte metadata backing with staged semantic indexes and copied
name bytes; source performs the entry/name/link-count/ctime/parent-time stores
directly. These independent components and the portable owner loans below use
the same byte layout. The subsequent native caller qualification executes the
source under the real kernel owner; Linux handler selection remains Stage 3.

The source validates extent, operation, time, selected object/entry associations,
regular-file type, actual versus stored link counts, initial-link eligibility,
clean destination capacity and staged input before mutation. Name handling
preserves raw bytes, rejects dot/dot-dot, NUL and slash, checks zero padding,
and copies/clears all 256 name bytes through ordinary checked unsigned codecs.
The owning controller still has to validate the complete filesystem, path and
reference admission, bind the inputs and independently check completion.

The exact action has **61 source functions**, within the unchanged 64-function
budget, and **89 native functions** including its retained runtime. Its complete
temporary bound is **57536 bytes**: source evaluation 57392, bridge 112 and
header 32, with one view and a maximum 73-bit operand bound. Its checked native
stack bound is **3824 bytes**, below the unchanged 4096-byte limit. The byte
component separately needs 17320 temporary bytes and 1248 stack bytes. Neither
source nor host resource limits were increased.

Rebuilding the exact source independently verifies the complete source object
and certificate. Altered instance, undercharged temporary bound, object bytes
and module snapshot are each refused. These checks bind the source component;
they do not substitute for the remaining native filesystem-controller binding.

The [independent C action oracle](labs/qemu/x86_64/namespace_action_client.c)
passes **8594 invocations**, comparing the entire 8192-byte backing against an
independent exact delta. It covers all 255 lengths at three guard placements,
every distinct object/entry pairing, raw high bytes, zero/maximum times and
full-width identities, replacement with retained links, unequal exchange counts,
rename with a full directory, descriptor/initial temporary linking, malformed
facts and selected records, each reserved input byte, readonly authority and
all 4096 short extents against inaccessible storage. Refusal cases use physically
readonly backing to detect stores before complete admission. The separate
[byte oracle](labs/qemu/x86_64/namespace_bytes_client.c) passes **8440 invocations**.
Observed arena peaks are 16672 and 10464 bytes respectively; those observations
do not replace either complete static bound.

The [GDB component oracle](labs/qemu/x86_64/namespace_action_debug.py) observes
four actual source calls, one per operation, and all **121 codec stores**.
It decodes the real arbitrary-precision offset/value operands, checks their
extent and zero stores to immutable staging, verifies the two-word frame and
revoked control, and confirms preservation of RBX, RBP, R12–R15, RSP and
XMM0–XMM15. The C caller checks physical arena/frame erasure. This does not prove
owning-kernel private-stack erasure, fatal completion or owner resumption.

Two separately built fault sources each pass another **8594 component
invocations**: one performs the mutations and returns false; the other returns
true after corrupting an entry association. Their actual stored bytes are
compared independently, and their source stack bounds remain 3824 and 3840
bytes respectively, with the same 57536-byte temporary bound. These witnesses
establish that neither false nor true completion alone guarantees safe owner
resumption. Together with the normal and byte components this is **34222 native
component invocations**, separate from the four GDB calls. The caller reclaims
its arena/frame; the production controller's fatal path and private-stack
erasure still require native integration and physical verification.

The [compiler checks](../src/topal-compiler/src/systems_namespace_action_tests.rs)
also prove that code sharing still charges every view creation: 80 helper calls
require 81 views and 6544 temporary bytes, while 256 helper calls exceed the
unchanged view limit and are refused. The
[frontend regressions](../src/topal-language/src/compiler_model/kernel_region_reuse_tests.rs)
check exact retention of smaller closures, original-graph refusal, distinct
declarations, every ordered call/value and the unchanged function budget.
Large analyzer match frames now return before ordinary recursive call analysis;
the existing 16 MiB Rust test-stack setting was retained. All **634 language
unit tests** and a full **268-test compiler run** passed before the native
comparison-helper test below was added. That new test passes separately;
this does not claim a full current 269-test compiler run. All four requirement/specification coverage
checks pass. These are separate validations, not a new complete workspace or VM
qualification.

## Stage 2: canonical owner and actual loans

The [metadata backing](../src/topal-semantics/src/systems_file_metadata_backing.rs)
now owns the actual encoded 4096-byte page from filesystem construction onward.
Every normal lookup, inode/stat observation, directory read, link count,
allocation-map read and page association comes from those bytes. The previous
parallel name/object/page-owner fields are removed. Inode and page-map actions
borrow the corresponding actual slices; descriptions and file payloads retain
their separate owners. Independent byte oracles preserve the existing page
record format: owner identity at zero, logical index at eight, then reserved
zero. Stored link counts must agree with the actual entry table. Raw geometry,
reserved/free bytes, object kinds, issued identities and description/reference
associations are validated before admission.

The new [namespace loan](../src/topal-semantics/src/systems_namespace_region_actions.rs)
checks the actual owner and copied names or a live descriptor, resolves the
operation, and stages inputs only after every recoverable check, including
nonwrapping invocation history. Same-inode no-ops issue no action. The opaque
view writes the actual owned page. Completion compares the entire page and
untouched input with an independent expected snapshot; that snapshot is never
copied into storage. Staging erasure and view retirement precede owner
resumption. Last-link victim reclamation occurs only after the namespace grant
retires and only without a remaining description. False completion, changed
association, missing reclamation or abandonment poison the owner, preserve
persistent stores and clear staging without rollback; terminal retirement erases
the complete page, payloads and references.

The [eight new ownership tests](../src/topal-semantics/src/systems_namespace_action_tests.rs)
compare all four transactions with the direct portable laws, inspect the exact
initial page independently, retain alias/data/position observations, exercise
nested/readonly/stale views and distinct same-label pools, and check descriptor
temporary linking, full capacity, no-op/exhausted history, overlapping controller
domains, malformed bytes/descriptions, thirteen failure/abandonment cases and
complete terminal cleanup. All **231 semantics unit tests pass**. These model
tests supply byte mutations and explicit logical cleanup obligations; compiled
source execution and physical private-stack reclamation remain separate proofs.

## Stage 2: native comparison and staging cleanup

The [native comparison helpers](../src/topal-compiler/src/systems_namespace_region_oracle.rs)
prepare an independent expected 4096-byte page and consume it during completion.
The independent component fixture surrounds comparison storage with guard
pages; the owning caller below separately binds production storage. Preparation
validates encoded metadata
geometry, free/reserved bytes, issued object identities, entry/link counts,
unique raw names and page associations, plus the staged operation inputs.
It writes only the private comparison page. A second preparation refuses while
preserving the live snapshot. These byte checks confer no ownership, reference,
credential or path authority and do not inspect separate payload/description
storage. The caller's pathname/reference admission remains a separate obligation;
the owning caller below binds the production code/data/storage layout.

Completion requires exactly true and compares every byte of the actual page,
including immutable staging, with that snapshot. It erases and reads back both
the 4096-byte comparison page and the 512-byte input staging on success or
failure. It preserves the actual persistent bytes, including deliberately
incorrect writes; the snapshot is never copied back. The helper returns false
on failure. The owning controller below retires the grant, erases source
temporaries and reaches its fatal halt before any owner resumption.

The [independent native client](labs/qemu/x86_64/namespace_oracle_client.c) links
these actual machine-code helpers with each compiled normal/false-after-store/
wrong-association source. Each variant passes **3650 source transactions**,
**839 preflight refusals**, **16384 whole-page byte corruption checks**,
**20054 non-destructive overlap refusals** and **20 invalid/empty completion
checks**. Preparation uses physically readonly owner backing at three guard
placements; inaccessible pages also surround the private comparison page.
The component's unnamed guard padding is a test fixture, not evidence of the
production storage binding below. Cases include every name length and
object/entry pairing, free object slots, alias/victim link counts, full directories, initial temporary
linking, maximum issued identities/clocks, resident page associations and
malformed maps/names/records. Native completion rejects false, non-Boolean and
true-without-effect results without rollback.

The [debugger observation](labs/qemu/x86_64/namespace_oracle_debug.py) separately
executes all four operations for each source variant. It observes eight actual
prepare returns and four completion returns per variant, independently derives
the complete expected snapshot, verifies unchanged owner bytes and live-oracle
preservation on overlap, and reads back all snapshot/staging bytes after
completion. It checks source false-after-store and wrong-association effects
before cleanup, persistent-byte preservation afterward, and RBX, RBP, R12–R15,
RSP and XMM0–XMM15 preservation. Actual emitted disassembly additionally checks
the leaf helpers' **8-byte stack cost**, with no local/red-zone stack accesses,
calls, callee-saved register writes or SIMD instructions. The
[compiler component test](../src/topal-compiler/src/systems_namespace_oracle_tests.rs)
preserves source records, objects, disassembly and separate stdout/stderr logs
for native, debugger and C compilation commands. This qualifies the helper
components. The distinct owning-kernel stack, abort recovery and physical
cleanup observations follow below.

## Stage 2: native caller, binding and abort recovery

The optional `namespace-library` declaration now passes through the shared
semantic model, root parser, public compiler pipeline, provider, ELF publication
and independent boot inspection. The initial native caller requires the existing
instance position, page and inode controllers. The
[root test](../src/topal-language/src/systems_source_tests_ram_pages.rs) admits
the complete declaration, refuses duplicate/malformed declarations and missing
controllers, and preserves the older root without the declaration. This selects
a separately bound controller capability; it adds no V6 syscall route and does
not reinterpret the primary V5 filesystem's entry consumers.

The [native caller](../src/topal-compiler/src/systems_namespace_region.rs) checks
the live fixed controller against the actual GS context, the exact metadata-page
location, private stack and masked interrupts, clean runtime, conflicting
file-controller domains and canonical staged metadata. It invokes the real
ordinary source on that page. A private 32-byte recovery record retains the
trusted caller frame, consumed epoch and original operation/time outside every
source view. History advances before entry, including when source later aborts.
Return checks the revoked control, history, frame facts and arena header before
complete byte comparison and retirement. Normal return preserves all GPRs except
the Boolean result and preserves SIMD; it clears the scalar frame, source
reserve, restored registers' popped spill slots and recovery record.

The [fatal recovery](../src/topal-compiler/src/systems_namespace_region_abort.rs)
reads its frame from that private record, independent of the aborting callee's
registers. It bounds the frame before writing through it, attempts all erasures
before any readback can skip another region, clears staging/oracle/arena/control/
recovery and the abandoned frame/return word, then reaches the bound fatal halt.
Persistent stores remain exact. Namespace history remains consumed; normal final
terminal retirement separately clears it after requiring all runtime idle.

The [native binding](../src/topal-compiler/src/systems_namespace_region_inspection.rs)
and [boot validator](../src/topal-compiler/src/systems_namespace_region_boot.rs)
bind all seven native functions, the complete source/runtime function closure,
immutable inputs, owner geometry, five private allocations and exact costs.
The new owner/invocation evidence schemas are each version 1. The common
storage inspector applies its existing alias/extent/section protections to the
additional 4096-byte comparison and 32-byte recovery allocations. The source's
3824-byte bound, 184-byte live caller and 64-byte alignment allowance total
**4072 bytes**. The qualified wrong-association source totals **4088 bytes**.
Both fit the unchanged **4096-byte complete stack limit**; source temporaries
retain the **57536-byte** bound within 65536. Native runtime functions do not
consume the separate 64-source-function qualification budget.

The added code/storage exposed a placement boundary: default linking placed
the 311296-byte process backing at 0x3d3000..0x41f000, crossing reserved user
addresses. The unchanged overlap guard rejected it. Selecting this controller
now places supervisor BSS at or above **8 MiB** and binds that placement;
filesystem/process extents and offsets remain the same. Roots without the
declaration retain their previous placement. No host memory, swap, security
or validation limit changed.

The [artifact tests](../src/topal-compiler/src/systems_namespace_binding_tests.rs)
publish the complete normal caller and both separately compiled fault sources.
They refuse **26** altered records, independently flip each of **96** bound
native/source functions while correcting the outer kernel hash, and reject
explicit null optional bindings. Production oracle/recovery storage remains
disjoint from all existing invocation stores. These are artifact/boot checks,
separate from executing the callback.

The [QEMU driver](labs/qemu/x86_64/namespace_controller_gate.py) and
[GDB inspector](labs/qemu/x86_64/namespace_controller_inspect.py) execute the
actual bound caller in ring 0 under its live controller. The debugger supplies
controlled canonical metadata and a native call before the prior personality
resumes. It verifies loaded native/source/data/caller code against the artifact,
checks complete 311296-byte owner snapshots and records every source codec
store. The normal four-operation run observes **121 stores**, preserved
registers/SIMD, epochs 1..4 across metadata reseeding, and physical erasure of
65536 arena bytes, 64 control bytes, 4096 comparison bytes, 32 recovery bytes,
512 staging bytes and **4272 stack bytes**, including popped spill slots.

Another **twelve fatal/refusal runs** cover a real allocator failure after a
persistent store, changed source frame/control, bad owner/view/input, exhausted
history, three conflicting file-controller states, and both actual fault-source
objects. Runtime abort succeeds even when callee-saved registers are deliberately
clobbered at the fatal hook. The false source returns zero after **39 stores**;
the wrong-association source returns true after **40 stores**. Each reaches the
bound fatal path with exact persistent writes retained and **4280 stack bytes**
erased, including the abandoned return word. Pre-entry invariant failures enter
no source. QMP independently confirms the target stays stopped after debugger
disconnect. This is **13 controlled VM observations / 16 controller calls**,
not Linux pathname/errno, final victim reclamation or unchanged user-program
qualification. Those remain the caller/consumer obligations in Stage 3.

## Stage 2 validation and review

The current working tree has **2341 passing Rust tests across all 41 workspace
test targets**, with no failed or ignored tests in the reconciled results.
This is a composite of completed targets from the initial locked workspace run
and a retry of the remaining packages. The initial run passed all **271 compiler
unit tests** and **261 compiler CLI tests**, then failed when LLVM could no
longer write temporary files because the memory-backed `/tmp` reached its user
quota. The failure logs remain retained; that first command is a failed
validation, not a successful full run.

The retry uses disk-backed scratch under ignored `linux-kernel/.cache`.
Three completed fixtures were moved there with links preserving their original
paths; all **183 files** retain their exact hashes. The retry passes all its
targets, including the failed region-action target, **635 language tests**,
**231 semantics tests** and all **four requirement/specification coverage
checks**. Reconciliation matches stable source identities against Cargo's exact
workspace target inventory, with no missing or extra targets. Package selection
changed dependency-feature unification for the retry; these composite results
do not replace Stage 4's fresh complete committed workspace qualification.

Locked workspace all-target checking, workspace formatting and direct formatting
of every changed Rust file pass. The direct check additionally covers included
fragments that workspace formatting does not traverse. The unchanged 2000-line
Rust ceiling, changed Python syntax and strict whitespace checks pass. All
**109 checked local Markdown targets** resolve, and the systems ledger retains
**310 unique rules**. Both public compiler publication and boot-packaging
commands accept the actual namespace declaration. Heavy validation uses the
unchanged bounded wrapper, its adaptive budget and normal host reserve, with
zero cgroup swap; no quota, memory or security setting was relaxed.

Risk remains high around actual-owner identity, replacement lifetime, opaque
loans, comparison-only state, abort recovery and physical retirement. Self-review
compares the authoritative resource/systems contracts, every affected model
consumer, independent guarded C/byte oracles, exact boot bindings and controlled
ring-0 GDB/QMP observations. No independent human review is claimed. The current
13-mode controller cohort uses one inspector revision and retains separate
stdout, stderr, serial/QEMU logs, complete owner snapshots and physical records.

## Remaining stages and initial acceptance

Stage 2's ordinary source, canonical owner loans, native binding and controlled
physical caller are implemented and component-validated. An encoded candidate
followed by native publication remains insufficient for subsequent kernel integration.
Stage 3 must implement all five Linux adapters, V6
dispatch with frozen V1–V5 controls, new versioned entry-table consumers, exact
source-driven effects, Linux/physical lifecycle/refusal cases and unchanged
BusyBox mv/ln. Stage 4 must execute the complete affected committed Rust/lab/
native/VM cohort and publish lossless evidence and accurate scope records.

The selected native kernel is unchanged from
[TK280](step-4-shared-inode-modes.md). Its 2305 Rust tests, 176 VM observations,
45 BusyBox workloads and 43 lab contracts qualify that distinct earlier native
checkpoint. The new controlled caller observations select no rename/link Linux
handler and qualify no new application workload. General directories/devices, whiteout, broader
credentials/VFS, allocation, dynamic userspace, distribution replacement,
containers and VM hosting remain part of the unfinished original goal.
