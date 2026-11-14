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

Stage 2 completed **2341 passing Rust tests across all 41 workspace test
targets**, with no failed or ignored tests in the reconciled results.
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

## Stage 3 in progress: V6 selection and pinned Linux baseline

The [V6 source](kernel/arch/x86_64/policy/static-linux-dispatch-v6.t) adds exactly
Rename, Link, RenameAt, LinkAt and RenameAtFlags to the unchanged 65 V5
operations, plus Unsupported. The shared selector model, library requirement,
frontend, numeric source bridge and independent boot-version checks recognize
this separate interface. It requires the actual namespace declaration and its
three controller dependencies. Declaring a callback with V5 remains a separate
component selection and does not select the new filesystem layout.

All five dispatch tests pass across V1–V6, including the actual native compiled
policies in original and reversed Enum order, holes through 65535 and full-width
inputs. V6 makes **131362 independent native selector calls** across its two
Enum orders. Its complete stack bound is **680 bytes** and its temporary bound
and observed high-water mark are **56 bytes**, within the unchanged limits.
All **232 semantics tests**, **636 language tests**, four coverage checks,
locked workspace all-target checking and the unchanged Rust size ceiling pass.
The policy-slot regression now enumerates all six dispatch versions, including
the previously omitted V5 bit, across all 2047 nonempty policy subsets.

The [Linux driver](labs/qemu/x86_64/namespace_linux.py) executes the
[raw-syscall controls](labs/qemu/x86_64/namespace_linux_guest.py) only in the
disposable, manifest-pinned Linux 7.2.9 VM. The first research run retained 61
observations and complete refusal-state checks; a second run asserts every
expected result explicitly and adds readonly-description and long-final-name
controls. The [66-case baseline](knowledge/x86_64/namespace-native-dispositions.json)
binds the kernel, manifest, actual guest/driver bytes and lossless guest,
serial/QEMU and readiness logs. It checks shared bytes/rights/positions, inode
metadata preservation, exact same-inode no-ops, replacement victims, exchange
with unequal link counts, full entry-budget moves, initial anonymous linking,
O_EXCL ineligibility and consumed link intent, 255-byte raw names, native casts
and distinct rename/link error orders. Every refusal preserves complete
observable namespace/inode/data/description state. This is a Linux reference
baseline; it qualifies no Topal Linux handler.

At the selection-foundation checkpoint, kernel publication of V6 failed closed
with an explicit unqualified entry-table error, covered by a compiler test.
The primary root remains V5; candidate V6 binding work follows below.
The complete new native consumers, all five adapters, source-driven lifetime
effects and unchanged mv/ln qualification still form the remaining cohesive
Stage 3 implementation. These foundation changes are not a completed Stage 3
or a fresh full-workspace/VM application qualification.

## Stage 3 in progress: native entry consumers and cleanup components

The new [private entry helpers](../src/topal-compiler/src/systems_namespace_entries.rs)
use separate entry slots and stable inode identities. A shared canonical-byte
checker validates the complete metadata page before lookup/free-slot selection;
operation-private directory staging may contain admission facts. The earlier
source PREPARE path retains its separate immutable-input checks. These private
helpers are included in the existing bound process entry; they add no source
callback, filesystem authority or persistent allocation.

The guarded [native C client](labs/qemu/x86_64/namespace_entries_client.c)
passes **1061 metadata checks and 6129 lookups**. It varies all four independent
entry/inode rotations and every raw length 1..255, with each input ending at a
guard page. Zero/256-byte lookup lengths use an unreadable pointer. It covers
multiple aliases, independent inode/entry exhaustion, invalid metadata and
duplicate raw names, large identities, nonzero private staging and terminal
retirement. Four retirement controls compare all 4096 metadata bytes against
an independent expected delta: remove every matching name, preserve every
unrelated name and clear only the selected inode's link count.

The [debugger control](labs/qemu/x86_64/namespace_entries_debug.py) observes the
actual emitted metadata leaf on valid and invalid input. It seeds distinct
nonzero caller-saved values, checks all fourteen other GPRs, restored RSP and
all sixteen XMM registers, preserves all metadata and the caller return word,
and observes physical zeroes in every **64-byte popped register spill**.
Disassembly establishes eight pushes/eight pops and no nested call, callee-saved
register use or SIMD instructions. Complete leaf/wrapper/lookup stack bounds
are **72/88/96 bytes**. The wrapper also retires its popped DI and completed
inner return word before checking success. The hosted component uses protected
read-only metadata and inaccessible owner/payload pages; it establishes no
live-kernel controller grant or complete V6 syscall path.

Versioned native consumers now select the new layout only for V6: lookup,
creation, reopening, unlink, directory enumeration, last-description release
and terminal retirement share the entry associations. Names are erased by entry
slot; an inode is reclaimed only after its last link and description. The new
volatile-file profile is version 6 with exact entry geometry and caller costs;
older profiles omit the optional record and retain their prior layout. Boot
admission checks the complete source/caller sum against the unchanged 4096-byte
reserve. These branches were unpublished at this component checkpoint: the
temporary V6 guard rejected kernel creation. Later candidate binding work below
replaces that refusal to permit VM qualification. The primary root remains V5.

The targeted native tests, previous comparison regression, frozen namespace
binding test, locked all-target workspace check and unchanged Rust size ceiling
pass. After the register-spill cleanup change, the native component was retried
successfully under the unchanged bounded wrapper, using about 2.7 GiB with
1569 MiB reserved for the host and zero cgroup swap. The debugger controls above
also pass. These component results do not claim a full current workspace run.

Before the parent-walker additions below, the locked compiler-library retry
completes with **273 passed,
zero failed and zero ignored**, in **630.05 seconds**, including every native
namespace/source/fault/binding test and the frozen dispatch controls. It uses
the unchanged bounded wrapper with a 2763 MiB adaptive budget, 1569 MiB host
reserve and zero cgroup swap. Complete stdout/stderr and command/result remain
in `linux-kernel/.cache/tk281-stage3-compiler-retry-1`. This is current working-tree
compiler evidence; Stage 4's complete committed workspace and VM qualification
remain required.
Final workspace/direct-included-file formatting checks, locked workspace
all-target checking and the unchanged Rust size ceiling also pass after this
retry. The changed Python parses, all 128 checked local Markdown targets resolve
and the complete diff passes the strict whitespace check.

## Stage 3 in progress: copied parent-path resolution

The [native walker](../src/topal-compiler/src/systems_namespace_walk.rs) now
separates copied-byte traversal from the preceding user-copy emission while
preserving their production order. V6's parent-only mode returns the actual
parent and final component offset/length/trailing facts before interpreting dot
components, imposing final length limits or looking up the final child. It still
resolves intermediate components normally. This supports rename's parent/mount/
write/error ordering without reusing open or unlink semantics. Root-only paths
retain their separate no-final-component disposition. Older profiles emit no
parent-only branch.

The [host adapters](../src/topal-compiler/src/systems_namespace_walk_component.rs)
execute that same copied-byte emitter against independently prepared guarded
buffers. The [C controls](labs/qemu/x86_64/namespace_walk_client.c) pass
**20494 entry-table cases and 12296 legacy cases**. Existing lookup/open/unlink
modes cover every final length 1..4090 with raw high bytes; the parent mode
defers those final lengths and separately rejects overlong intermediate names.
Literal controls cover live/missing names, immutable parents, dot/dot-dot,
trailing slashes, intermediate regular/missing objects, root-only paths and
all three supported working directories. Absolute paths ignore an invalid
dirfd; relative paths exercise native AT_FDCWD with nonzero upper bits.

The copied path and canonical metadata are read-only during execution; every
case compares complete pages and permits changes only to the five walker fact
words. Only the prefix/fact, copied-path and metadata pages are accessible;
remaining owner/payload pages stay inaccessible. Real descriptor and
user-copy services are outside this component: an attempted descriptor lookup
traps, and user-copy code is excluded. The first harness attempt included that
code and was correctly rejected by the unchanged pure-component instruction
verifier; its logs remain retained. Separating emission allowed the actual walker
component to pass that verifier without weakening its rules. These observations
qualify copied-path traversal, not Linux syscall ordering or a live V6 kernel.

V6 selection now requires the existing `file-open-decision-v1` kernel policy as
well as the namespace/controller dependencies. This carries the named-file
profile's open-plan requirement into the new mutable layout: creation, reopen,
truncate and unlink cannot silently fall back to the boot-only policy. The
systems design, formal ABI rule, tool requirements, frontend and shared semantic
validation are aligned. Missing and boot-only policies have explicit refusal
tests; older dispatch and standalone namespace components retain their selection
rules. All five adapters and their physical/source/utility qualification remain
unfinished at this checkpoint, with the temporary V6 refusal still in place.

After these additions, all **232 semantics tests**, **636 frontend tests** and
**15 namespace-filtered compiler tests** pass, with no failed or ignored tests.
The compiler subset includes normal/fault source bindings, exact owner/stack
admission, independent byte oracles, native entry/debugger controls and both
copied-walker layouts. The separate V6 publication-refusal test and all **four
requirement/specification coverage checks** also pass. Workspace all-target
checking, workspace/direct included-file formatting and the unchanged Rust size
ceiling pass under the normal bounded wrapper. Complete command/result and
stdout/stderr logs remain in
`linux-kernel/.cache/tk281-stage3-parent-integration-1`. All **156 checked local
Markdown targets** resolve, changed Python parses and strict whitespace checks
pass. The primary V5 root/policy files remain untouched. This is affected
working-tree qualification; the complete current 274-test compiler suite,
committed workspace and fresh complete VM/application cohort remain Stage 4
obligations. Stage 3 still requires the five adapters and their Linux, physical
and unchanged-utility qualification before its cohesive commit.

## Stage 3 in progress: native adapters and source handoff

The new [adapter emitter](../src/topal-compiler/src/systems_namespace_transactions_native.rs)
normalizes rename, link, renameat, linkat and renameat2 arguments, including
native low-32-bit flags and independently checked old/new dirfds. Selection is
V6-only; the primary root and older dispatch branches remain unchanged.
[Rename admission](../src/topal-compiler/src/systems_namespace_rename_native.rs)
resolves both parents, distinguishes the shared readonly root/bin mount from
the mutable mount, and orders final type, write, lookup, NOREPLACE/EXCHANGE and
trailing-name checks. Same-inode cases skip the source action completely.
[Link admission](../src/topal-compiler/src/systems_namespace_link_native.rs)
resolves its old path/description first, then checks destination existence,
trailing name, write and mount disposition before inode eligibility/capacity.
This keeps anonymous-file eligibility behind destination errors. Whiteout and
other unqualified inode kinds never become ordinary successful operations.

The [commit helper](../src/topal-compiler/src/systems_namespace_transaction_commit.rs)
keeps all eight checked plan words in registers across the clock, serializes
the existing input representation and copies the new raw name with complete
padding. It retires copied pathname and transfer staging before calling the
actual ordinary namespace controller. It performs no native link/rename stores
to persistent metadata. After the loan has retired, replacement-victim cleanup
checks remaining names and descriptions before releasing object/pages. Completed
inner/owning return words are erased. The added owning helper return word fits
the declared 256-byte caller allowance; source/caller totals remain 4080/4096.

V6 recoverable and no-op exits restore the trusted captured kernel-entry stack
position, bound/alignment check it and erase/read back its retired 4096-byte
call area. Completed clock calls are retired before source entry or a clock
error. The versioned profile now binds the **104-byte nested lookup maximum**
and **4096-byte admission retirement** separately from source/caller costs.
The direct metadata leaf/wrapper/entry-helper bounds remain 72/88/96 bytes.

The code-emission test constructs the complete V6 process personality directly
through the private emitter, resolves every local branch and records exactly
one namespace-controller handoff relocation. It preserves the object and
external-call record without publishing or booting an unqualified kernel.
The guarded [final-name component](labs/qemu/x86_64/namespace_final_client.c)
executes those same final-kind/lookup helpers: **4099 classifications and 12300
lookups** pass, including dot/root deferral, immutable names, independent
mutable entry/object slots and every copied length through 4095. Long-name
classification uses an unreadable pointer to prove that it defers byte access.
Metadata is read-only and unchanged; the pure-component verifier is unchanged.
These are final-stage component checks, not execution of complete syscall paths.

Cleanup review found an advancing-DI readback error in the new erase usage:
readback started after the erased bytes. The helper now rewinds DI before
verification. A new native regression places 512 nonzero bytes at a page end,
protects the following page, invokes the same erase emitter and checks exact
prefix preservation and complete retirement. The entry/debugger and both
parent-walker cohorts also pass with this correction.

All-target workspace checking, emitter closure, final-name components, guarded
entry/erasure/debugger components, both parent-walker layouts, the publication
refusal and unchanged Rust size ceiling pass. Separate stdout/stderr and command
results remain in `linux-kernel/.cache/tk281-stage3-adapter-checks-2`.
The earlier emission/cohort is retained separately as
`linux-kernel/.cache/tk281-stage3-adapter-checks-1`. No current full 276-test
compiler or new V6 VM/application qualification is claimed.
All four requirement/specification coverage checks also pass after the cleanup
refinement; all 165 checked local Markdown targets resolve, changed Python parses
and the complete diff passes strict whitespace checking.

At this emission checkpoint the temporary guard still rejected V6. The five
adapters had not executed as complete Linux entries. Fatal source recovery also
needed to retire the new outer commit return frame before halt, while preserving
the older component profile. Subsequent binding/VM work is recorded below.
That integration, exact new bindings, pinned Linux
comparisons, physical failure/lifetime/cleanup observations and unchanged mv/ln
remain required before the cohesive Stage 3 commit.

## Stage 3 in progress: V6 binding and real syscall execution

The V6 fatal continuation restores only the protected kernel-entry stack
context, validates its bounds/alignment and erases and reads back the complete
4096-byte abandoned admission area, including the owning commit return word.
Invalid stack contexts never authorize a write. Earlier recovery attempts all
its owned cleanup before verification; persistent source effects remain on
fatal exit. The final halt occurs after cleanup and register clearing. V6
selects this continuation through owner/invocation binding version 2, with
exact retirement geometry and halt offset. Version 1 templates/bindings retain
their earlier behavior.

Public candidate/boot admission checks the new binding and rejects 16 altered
records plus explicit null retirement. Normal source plus the complete caller
costs **4080 bytes**; the wrong-association variant reaches **4096 bytes**.
Invocation-frame accounting remains separate from the additional owning return
word. Both old and V6 bindings admit actual false-after-store and
wrong-association source objects, retaining their complete linked/boot fixtures.
The four-source admission test passes in
`linux-kernel/.cache/tk281-stage3-v6-fault-bindings-1`.

The development guard has been removed to execute private V6 candidates through
the public compiler. The primary root still selects V5. Provider/artifact
versions are 241/254. This is uncommitted Stage 3 qualification, with no final
application or PR acceptance claim.

The [transaction caller](labs/qemu/x86_64/static_namespace_transaction_checks.py)
first passed **49 actual syscall returns**, covering all five entries, native
upper-bit adaptation, alias counts, identity/data/rights/position, same-inode
no-ops, open replacement-victim lifetime and anonymous initial-link eligibility.
The failed first attempt exposed a caller defect: its inherited encoder emitted
four registers while recording a fifth flags argument. A local six-register
encoder with matching return PCs corrects that defect. Both attempts remain in
`linux-kernel/.cache/tk281-stage3-v6-syscalls-1` and `-2`.

The [disposition caller](labs/qemu/x86_64/static_namespace_disposition_checks.py)
then passed **294 observed returns**, including **45 explicit error-ordering
cases** checked against the retained pinned Linux baseline. The readonly
reference file is represented by the immutable `/bin/busybox` object, with
matching writable/readonly mount dispositions. Four descriptions' complete
stat records are compared after each refusal; final content/position checks
also complete. This does not claim an identical general filesystem topology
or all 66 reference events. A further 294-return run adds read-only physical
observation. Across 59 namespace entries, 52 refusals/no-ops preserve all
311296 bytes of owned process/filesystem backing and the namespace generation.
Seven mutating entries each invoke the actual controller exactly once.

The extended caller adds distinct-inode exchange with unequal link counts,
full name-table rename, independent inode capacity, closed replacement-victim
reclamation and 255-byte raw filenames. It passes **350 total returns**.
Its [physical observer](labs/qemu/x86_64/namespace_syscall_physical.py) observes
**66 namespace entries**: **53 refusals/no-ops preserve the entire owner and
generation**, with no source entry; **13 mutations use one source action each**.
All observed namespace returns have zero source private buffers, copied path,
transfer staging and directory input staging. The observer verifies actual
user argument registers and SYSCALL instructions and injects no kernel state.
Successful debugger detachment and completed user assertions/serial markers
are required before the final result is accepted.

Lossless outer command/stdout/stderr, VM/debugger logs and hash-bound records
remain in `linux-kernel/.cache/tk281-stage3-v6-dispositions-1`,
`linux-kernel/.cache/tk281-stage3-v6-refusals-physical-1` and
`linux-kernel/.cache/tk281-v6-extended-1`. At this 350-return checkpoint, exact
source-store deltas and real-entry stack retirement remained separate work;
the subsequent run below adds those observations.

All **13 V6 controlled controller modes** pass with QMP confirming a stopped
target: normal, allocator abort, post-frame/control faults, actual source false
and wrong association, and seven pre-entry owner/view/input/generation/overlap
refusals. Fatal observations include the additional owning return-word marker
and the whole 4096-byte abandoned admission area. Five results are in
`linux-kernel/.cache/tk281-stage3-v6-controller-cohort-1`; the remaining eight
are in `linux-kernel/.cache/tk281-v6-ctl-2`. The first wrong-association launch
failed before boot because its Unix socket pathname exceeded the host limit;
the retained retry uses a shorter output path. These remain controlled direct
controller calls, separate from real user syscall qualification.

The latest run passes **465 syscall returns**, retaining all preceding cases
and filling the entire eight-page file budget after a closed replacement
victim is retired. The surviving source owns one page and three new files use
the other seven pages. An initial test incorrectly expected a large write to
complete in one call; the corrected caller uses the existing 256-byte write
quantum and checks each file's final size. The failed attempt remains in
`linux-kernel/.cache/tk281-v6-extended-pages-1`; successful lossless evidence is
in `linux-kernel/.cache/tk281-v6-extended-pages-2`.

The new [source observer](labs/qemu/x86_64/namespace_source_physical.py) records
**461 actual ordinary source stores** across the 13 mutating calls and all
four actions. It checks the source frame, generation, oracle bytes and actual
plan, replays every store against the complete owner, and independently
compares the final owner delta before native victim reclamation. The ordinary
source and owner returns preserve their declared GPRs/SIMD. Every source return
has its 4272-byte frame/retired area erased; every one of the 66 namespace user
returns has the entire 4096-byte owning admission area erased. No debugger
state injection is used for these real syscall observations. At this checkpoint,
actual syscall fatal paths, clock-failure, broader lifetimes and unchanged
utilities still required separate qualification; the next section records
subsequent fatal, clock and directory-consumer observations.

Workspace and included-file formatting, locked all-target workspace checking,
the **full 277-test compiler suite**, all **four coverage checks**, and the
unchanged 2000-line Rust ceiling pass. The compiler suite completes in 712.51
seconds. Of **43 lab contracts, 41 pass**: the two remaining failures detect
the old checked-in gate schema and inode gate-script hash. Those retained
records must be regenerated by the complete fresh qualification before
publication; their expected hashes or assertions have not been weakened.
Logs remain in `linux-kernel/.cache/tk281-stage3-retry-validation-1` and
`linux-kernel/.cache/tk281-stage3-retry-sizes-1`. All commands use the unchanged
resource wrapper, normal host reserve and zero validation swap, with no memory
limit failure. The complete committed Stage 4 cohort remains unfinished.

## Stage 3 in progress: real syscall failure and consumer qualification

The [fatal caller](labs/qemu/x86_64/static_namespace_fault_checks.py) prepares
one admitted operation through each actual Linux entry: path link, initial
anonymous-file linkat, move, replacement through renameat and exchange through
renameat2. The public compiler and boot builder produce each private candidate.
Two variants contain real ordinary source that returns false after stores or
writes a wrong association after an otherwise correct action. The allocator
variant uses the normal source and exhausts its actual arena after the first
store, taking that source's linked abort hook at its next allocation.

All **15 combinations** pass the
[real syscall fatal observer](labs/qemu/x86_64/namespace_fault_inspect.py).
It verifies actual CPL3 arguments and the SYSCALL instruction, complete bound
code/data, the real owner and two-Nat frame, source stores, consumed generation
and exact complete-owner effects. Both genuine abandoned return-address words
are observed before recovery and read back as zero. At the final fatal boundary,
the **4280-byte source frame/reserve**, **4096-byte owning admission area**,
all private runtime/input/path/transfer bytes and GPRs are erased. Persistent
effects remain exact; no owner or user return occurs. The debugger disconnects
and QMP independently confirms the target is stopped.

The false/wrong source variants use **no debugger state injection**. Allocator
failure explicitly injects arena exhaustion and clobbers saved GPRs at the
actual noreturn abort hook to test recovery from its protected record. Exhaustion
after the first store fails at the next allocation: replacement can complete
a second constant-only store first. An initial observer incorrectly required
exactly one store at abort; it now records and verifies every completed store.
The failed attempt is retained. Successful store counts are 39/39/36/40/6 for
false results, 40/40/37/41/7 for wrong associations, and 1/1/1/2/1 for allocator
abort across link/linkat/move/replace/exchange.

Thirteen successful observations and the failed replacement attempt remain in
`linux-kernel/.cache/tk281-v6-fatal-suite-iy2k8a86`. Source snapshots preserve
the exact earlier observer hashes before that assertion correction. The last
two successful allocator observations and a fresh **465-return normal physical
regression** are in `linux-kernel/.cache/tk281-v6-fatal-resume-c6j_isw4`.
The normal regression again observes all 461 stores, exact owner deltas and
source/admission retirement. No kernel code changed for these tests.

The [clock observer](labs/qemu/x86_64/namespace_clock_inspect.py) binds the full
native process function and identifies the actual RTC status-read instruction
from the linked ELF. A read-only run of the existing 465-return image confirms
that all **53 refusals/no-ops perform no RTC access and issue no source action**;
the 13 mutating calls read the clock before their source action. Five separate
callers force the guest's update-in-progress bit at that actual input instruction.
Every entry executes all **1024 bounded retries**, returns `EIO`, preserves the
complete 311296-byte owner and namespace history, issues no loan, and erases
private buffers plus the 4096-byte admission area. User assertions, successful
debugger detachment and serial completion must all finish. This VM input fault
changes no host setting or resource allowance. Full raw evidence is in
`linux-kernel/.cache/tk281-v6-clock-suite-r2385fr0`.

The [directory caller](labs/qemu/x86_64/static_namespace_directory_checks.py)
passes **64 real syscall returns** after arranging entry slots 0/1/2 to name
inode slots 2/0/1. It checks raw directory records, shared inode identities,
entry-slot cookies, short-buffer and failed-copy cursor preservation, sparse
entries after unlink, and fresh enumeration after exchange. Path/descriptor
chmod, fstat/fstatat, reopen/read and last-description cleanup continue to use
the correct inode after names exchange. The same physical source observer
checks the link/exchange actions and temporary retirement. Artifacts, complete
logs and the separately bound caller record are in
`linux-kernel/.cache/tk281-v6-directory-yfasg_y_`.

These remain Stage 3 candidate observations. Shared namespace history and inode
lifetime through fork, exec and inactive cancellation, unchanged BusyBox mv/ln,
primary-root activation and the complete committed Stage 4 evidence refresh
were still required at that checkpoint; the following observations qualify
the lifecycle and utility paths. The committed Stage 4 evidence refresh remains
required.

## Stage 3 lifecycle and unchanged utility qualification

The [fork caller](labs/qemu/x86_64/static_namespace_fork_checks.py) exposed a
real consumer defect: the older inode-mode native guard rejected a valid inode
with two names. The first run stopped on the child's chmod after nine observed
returns and timed out; its artifacts remain in
`linux-kernel/.cache/tk281-v6-fork-first-1`. V6 now selects inode invocation
schema 2, validates the complete canonical metadata through the existing closed
read-only leaf, accepts zero through four links and refuses overlapping
position/page/namespace loans before source entry. Its added leaf requires
72 complete stack bytes and erases both popped spills and the completed return
word. Sequential admission and source paths retain the **1544-byte** maximum.
Earlier selections retain their exact schema 1 code and omitted optional field.

Current provider/root artifact versions are **242/255**. Exact V6 binding tests
reject seven altered inode admission records and explicit null, in addition to
the earlier namespace refusals. The legacy inode binding control, locked
all-target workspace check, formatting, unchanged Rust ceiling and four coverage
checks pass in `linux-kernel/.cache/tk281-v6-inode-admit-w2phln00`.

The corrected fork candidate passes **37 real returns**. The child adds an
alias, renames and unlinks names, changes shared permissions, writes through a
duplicated description and exits. The parent waits, checks status, identity,
rights, shared/independent positions and both payloads, then mutates and retires
the remaining names. Explicit protected-history checkpoints prove epochs
1/2/3/3/4/5/5 without rewind across parent restoration. A fresh **465-return**
normal physical regression and **64-return** directory regression also pass on
this native provider. All three are in
`linux-kernel/.cache/tk281-v6-fork-positive-9hvrjiaw`; the exact earlier observer
version is retained there by its recorded hash.

Two [physical inode-admission faults](labs/qemu/x86_64/namespace_inode_guard_gate.py)
follow the actual child's chmod lookup. An injected three-link count with only
two associations reaches the metadata checker and is refused; a marked-active
namespace control is refused before that checker. Neither enters ordinary mode
source or resumes user code. The complete injected owner and all captured runtime/
control bytes remain exact; the executed checker retires all 72 stack bytes.
QMP confirms stopped targets. These VM-only fault injections and the complete
**277-test compiler pass** (711.39 seconds) are in
`linux-kernel/.cache/tk281-v6-fork-guard-_xocn51e`.

The [exec caller](labs/qemu/x86_64/static_namespace_exec_checks.py) passes
**39 real returns** through the existing three successful same-image execs.
It verifies CLOEXEC closure, retained independent reader/shared writer positions,
aliases, identity and link counts. The entire **36864-byte filesystem** is
captured before the first successful exec and compared byte-for-byte afterward.
Explicit epochs stay at two across that exec and advance to three before the
later two execs. After final reference release, a fresh inode has identity eight
rather than reusing seven; its link and rename advance history to four/five.
No guest state is injected. Artifacts, snapshots and physical source evidence
are in `linux-kernel/.cache/tk281-v6-exec-44j8tkxt/candidate`.

The [cancellation caller](labs/qemu/x86_64/static_namespace_cancel_checks.py)
uses actual fork, child sched_yield and parent exit instructions, without
debugger register or memory edits. It passes **22 ordinary returns** and three
observed source actions. Before exit, the parent observes the child's shared
payload, permissions, positions and two aliases while the child remains parked.
At namespace finalizer entry, both descriptor banks, names, inode, pages and
the complete **311296-byte owner** have been erased; history is still three.
The finalizer resets history to zero, and the native process controller returns
with the complete owner and namespace runtime still zero. Neither parent exit
nor child yield resumes user code. Full before/after snapshots, debugger logs,
physical source evidence and QMP stop confirmation are in
`linux-kernel/.cache/tk281-v6-cancel-first-1`.

The [utility gate](labs/qemu/x86_64/namespace_utility_gate.py) passes six fresh
workloads using the exact unchanged pinned BusyBox ELF: mv to an absent name,
ln with equal reported link counts, replacement with a retained victim reader,
chmod through an alias, unlink of both names with a retained reader, and relative
ln/mv followed by directory enumeration. Every run checks exact output, exit
status and successful exec count. These functional application observations use
the same real services as the separate physical callers; no command-text rule
selects namespace behavior. Artifacts and complete serial logs remain in
`linux-kernel/.cache/tk281-v6-utilities-first-1`.

The primary source now selects V6 and the namespace library. Existing lab
consumers accept that explicit new vocabulary, dispatch qualification retains
all five frozen older selections, and the native-only page absence control
explicitly selects V5 without the namespace source dependency. Six utility
commands have been added to the primary qualification workload set. These
changes require fresh complete Stage 4 publication records; old checked-in
records and hashes are not substitutes. All heavy commands retain the normal
resource wrapper, host reserve and zero validation swap.

The primary clean boot smoke passes on the new root. After activation, all
**636 language tests** and the complete **277 compiler tests** pass; the latter
finishes in 978.47 seconds. The first language run exposed one test still
replacing V5 selector text in the now-V6 primary root. Its failed 635/636 result
is retained; the corrected control explicitly selects each frozen version and
also tests V6. Page/position absence, provider relocation and structural-plan
controls likewise explicitly retain their original profiles. Current successful
suite logs are in `linux-kernel/.cache/tk281-stage3-primary-validation-2`.
The current inode publication contract requires a fresh V6 consumer archive
under `results/tk281/inode-consumer`; the historical TK280 archive remains intact.
Final workspace and included-file formatting, locked all-target workspace
checking, all four coverage tests, the unchanged Rust source ceiling and
whitespace checks pass. All **39 changed Python files** parse and all **194**
checked local documentation targets resolve. Complete final-check logs are in
`linux-kernel/.cache/tk281-stage3-primary-finish-u316ia6u`. These checks complete
the integration stage; full committed qualification and publication records
remain Stage 4 work.

The first committed Stage 4 workspace run, at `88415f0`, completed with **2346
passing tests and three fixture failures across 41 targets**. Locked all-target
checking and the separate doc-test command passed; no memory limit failed.
The packaging fixture omitted the newly required namespace snapshot and supplied
V5 dispatch to the V6 root. Its independent source closure now explicitly supplies
V6 and both namespace helper modules. The four-instance RAM-page and legacy
unframed-position controls now explicitly select their original profiles before
removing or changing source declarations. Their original bounds, isolation,
refusal and native-service assertions remain intact. All **10 targeted builder
tests** pass after those corrections.

The new qualification wrapper passed a real primary boot and retained its
complete kernel/provenance/serial/QEMU artifacts on repository disk after the
temporary context exited. A short owned `/tmp` alias keeps debugger socket names
within their platform limit. The initial queued cohort used a filename already
assigned to the retained-identity gate; that gate was restored exactly, the
wrapper received its own name, and the queued attempt failed argument parsing
before any VM ran. The failed setup record remains separate from kernel results.
The complete workspace failure and original fixture snapshots are retained in
`tools/rust-qualification/results/tk281/before-builder-fixture-fix`.
Fresh committed full qualification is still required before publication.
The corrected integration preflight again passes formatting, locked all-target
workspace checking, all four coverage checks, the unchanged Rust ceiling and
whitespace checks. All **49 Python files** in the complete integration diff
parse. The retained runner and archive tools are included with the integration
inputs so the next qualification can bind one committed revision.

## Stage 4 committed qualification refresh

The corrected committed revision `9e7875eb9731f231d74ed45e6e4c5b34efefddee`
passes locked all-target workspace checking, all **2349 implementation tests
across 41 targets** with no failures or ignored tests, and the separate doc-test
command (zero executable doc tests). The
[verbatim Rust record](tools/rust-qualification/results/tk281/final/qualification.json)
and lossless logs bind the actual source tree, tool identities and unchanged
bounded wrapper. The earlier three-fixture failure remains separately archived;
its successful check and doc commands do not replace the corrected full run.

The completed affected VM refresh passes **230 observations**: 117 baseline,
54 application/environment, 47 namespace and 12 inode dispositions. All **45
lab publication contracts** pass before and after external-dependency
reconstruction. The [final manifest](labs/qemu/x86_64/results/tk281/final/qualification.json)
binds the real command logs, original inputs, current namespace/inode records,
verification revisions and publication audit. Its 11272 logical artifact files
retain their exact original bytes and hashes.

The fresh application corpus passes all **51 unchanged pinned BusyBox workloads**
with exact output, status and exec-count checks. Together with clean primary
boot and empty/null environment controls, it executes **54 actual VM runs**.
Per-execution serial and QEMU log paths and hashes bind each workload to its
real observation. The fresh V6 inode suite also passes all **12 physical
dispositions**, including lifecycle cleanup, seven pre-entry refusals and two
post-store source failures. Its
[current consumer archive](labs/qemu/x86_64/results/tk281/inode-consumer/archive.json)
retains the original machine records, five unchanged BusyBox mode observations
and **46 exact compressed machine/application logs**, bound to the completed
Rust run and current primary kernel. These dispositions are included in the
230-observation total; they are not additional observations.

The first baseline refresh passed its RAM-page, file-position-source and root-region
groups, then stopped in the first positional-file observer. Its legacy assertion
requires bytes 64–255 of filesystem metadata to be zero; V6 places independent
entry associations in bytes 64–127. The named and anonymous file observers
contained the same old layout assumption. This was an observer assertion failure,
not a memory-limit failure. The failed command, original sources and complete
artifacts remain in `linux-kernel/.cache/tk281-s4-baseline-1` and the
[historical archive](labs/qemu/x86_64/results/tk281/before-v6-consumer-fixture-refresh/baseline-and-observer-preflight/archive.json).

An independent V6 store checker now passes a component test for rotated slots,
shared aliases, anonymous/unlinked lifetimes, malformed entries and inodes,
exact link counts, page associations, payload tails and complete retired-byte
zeroing. It consumes explicit versioned artifact geometry and checks its own
source digest. Corrected observer branches were prepared separately while the
existing namespace cohort retained its frozen sources. That cohort completed
all **42 stages / 47 VM observations**, including all fifteen fatal source
cases, clock failures, admission refusals and lifecycle/utility checks; its
[verbatim archive](labs/qemu/x86_64/results/tk281/before-v6-consumer-fixture-refresh/namespace-consumer/qualification.json)
retains those exact earlier inputs.

The installed observer corrections pass **12 actual VM dispositions**: four
each for positional, named and anonymous files, including lifetime/termination
and deliberate payload-corruption controls. Legacy observer branches retain
their existing assertions. No compiler or kernel source changed; the completed
Rust run still qualifies that committed code. The completed VM refresh binds
the updated observer snapshots and the additional component test, bringing the
required lab publication count to **45**. All four VM cohorts and those
contracts pass.

The first complete publication-contract run exposed three old schema
expectations and a stack check applied to source refusals that never created
an admission frame. Corrected checks require the exact V6 geometry, require
4096-byte retirement for admitted source and clock-error frames, and require
no frame plus complete owner/history preservation on unentered refusal paths.
A subsequently reached test-variable collision was also corrected. Both failed
verification runs remain retained.

The [verification revision](labs/qemu/x86_64/results/tk281/namespace-consumer/verification-revision.json)
binds exactly the two publication-contract files and the post-run archive
verifier separately from the unchanged experiment source snapshots. Original
snapshots and VM outcomes are never relabeled. The revision validator refuses
runtime-harness changes, damaged original snapshots and unrecorded verifier
edits; both negative/round-trip tests pass. Kernel/compiler source and every
executed VM input retain the completed run's provenance.

Raw VM artifacts can include the pinned external BusyBox ELF and generated
initramfs files containing it. Under [TK-BINARY-001](design/license-and-provenance.md),
the [publication tool](tools/archive-publication/README.md) keeps those blobs
in the ignored external cache. Original producer records and indexes remain
verbatim; the published index retains every artifact's original length and
digest, with exact reconstruction recipes using the pinned external input and
first-party wrapper bytes. No observation or dependency entry is skipped.
All three storage-publication tests pass, including complete recovery after
cache loss and refusal of altered inputs, recipes and destinations. Current
and historical archives have been projected and hydrated successfully. All
45 contracts pass afterward. The completed binary-boundary audit checks all
6520 prospective published gzip objects and finds no complete external BusyBox
ELF inside Git-bound artifacts.

## Completed increment and remaining initial acceptance

Stage 2's ordinary source, canonical owner loans, native binding and controlled
physical caller are implemented and component-validated. An encoded candidate
followed by native publication remains insufficient for subsequent kernel integration.
Stage 3 integrates and qualifies all five Linux adapters, V6
dispatch with frozen V1–V5 controls, new versioned entry-table consumers, exact
source-driven effects, Linux/physical lifecycle/refusal cases and unchanged
BusyBox mv/ln. Stage 4 completes the affected Rust/lab/native/VM qualification,
lossless evidence and accurate publication scope for this increment.

The preceding selected native kernel is documented in
[TK280](step-4-shared-inode-modes.md). Its 2305 Rust tests, 176 VM observations,
45 BusyBox workloads and 43 lab contracts qualify that distinct earlier native
checkpoint. The earlier controlled components select no rename/link Linux
handler; the V6 candidates execute the actual entries and six unchanged
application workloads. Current complete affected publication evidence is linked
above. General directories/devices, whiteout, broader
credentials/VFS, allocation, dynamic userspace, distribution replacement,
containers and VM hosting remain part of the unfinished original goal.
