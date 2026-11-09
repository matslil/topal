# Readiness for general kernel allocation and storage

This inventory connects `TK-ELEMENT-STORAGE-001` and `TK-KERNEL-MEMORY-001`
to the qualified implementation at compiler/kernel commit
`9e7875eb9731f231d74ed45e6e4c5b34efefddee`. It identifies work required beyond
the current RAM-file and namespace profiles. New admissible systems profiles
need their own exact rules, source/native binding and execution evidence.

## Existing language meaning

Ordinary Topal already defines terminating recursion, including mutually
recursive declaration groups, constrained-integer and structural descent,
and checked `Decreases` evidence. Every recursive cycle must decrease a
well-founded measure. See [execution](../../../docs/execution.md) and
[capabilities](../../../docs/capabilities.md). A runtime iteration counter alone
does not discharge that source-language termination obligation.

The [physical-frame contract](../../design/physical-frame-allocation.md)
already defines nonzero counts, power-of-two alignment, distinct live affine
extents, non-overlap, allocator identity and whole-extent release. Allocation
policy and free-range structures belong in ordinary Topal libraries. Numeric
frame indexes and copied metadata remain facts; live allocator and extent
identities carry authority.

The [borrowed-region contract](../../design/borrowed-kernel-regions.md)
already permits ordinary source to operate on actual exclusively owned bytes.
It keeps owner access suspended, checks view/frame/generation identities and
rights, preserves admitted persistent writes on failure, and requires complete
temporary reclamation before owner resumption. Source cannot create authority
from numeric addresses or use a temporary arena as persistent storage.

## Qualified implementation limits

| Facility | Current qualification | Required generalization |
| --- | --- | --- |
| Physical extents | Selected static counts and separately qualified coexistence profiles | Requests derived from live workload needs, multiple independent live extents, reservations, exhaustion and reuse across the actual usable memory map |
| Ordinary source calls | Closed acyclic helper graphs; the numeric compiler rejects recursive closures | Qualified ordinary iteration/recursion with proven termination and complete native resource bounds |
| Source temporary storage | Exclusive 65536-byte arena and 4096-byte stack, with full invocation erasure | Proven bounded live temporary storage for repeated work; any reuse must preserve still-live values and borrowed authority |
| Persistent filesystem storage | Four independent regular inodes, four entries and eight reusable resident pages | Storage capacity derived from owned memory, hierarchy and object lifetimes, without fixed application-specific slot limits |
| Persistent source values | Separately qualified retained numeric/structured graphs and borrowed-byte actions | Allocation and reclamation for general live subsystem objects, with explicit ownership and publication |
| Shared use | Selected synchronous operations and explicit peer-loan exclusion | Qualified synchronization, preemption and multiprocessor ownership protocols |

The ordinary source restrictions are documented in the
[systems profile](../../../docs/systems-profile.md), and the recursive-closure
refusal is implemented in
[numeric library analysis](../../../src/topal-compiler/src/systems_numeric_library.rs).
The [kernel status](../../step-4-status.md) records the current executed
profiles and the full distribution/container/VM-host obligations.

## Evidence needed for the next storage implementation

First establish which existing ordinary termination, function/effect and
resource analyses can qualify the allocator's actual source. Preserve the
language's arbitrary-precision arithmetic, effect order and borrowed-region
meaning. A qualified iterative or recursive lowering must bind its complete
closed source graph, progress proof, actual inputs and native ABI. Its stack
and allocation certificate must cover every admitted execution; observed high
water marks alone are insufficient.

Reusing a physical frame or source temporary area requires proof that its old
contents and references have retired. Function-call optimization must preserve
logical view/frame lifetimes, fresh identities, generation monotonicity,
callee code binding and source-level behavior. The owning invocation resumes
only after its complete cleanup obligations hold. Failure after actual
persistent stores keeps those stores and reaches the existing fatal boundary.

Allocation must act on the real owner and return extents with provenance.
Tests need exhaustion, fragmentation, alignment and overflow cases; disjoint
simultaneous allocations; foreign, stale and duplicate release; reservation
preservation; zero-before-republication; repeated reuse and terminal cleanup.
Independent complete-state oracles and physical observations must distinguish
ordinary source effects from native admission and machine transitions.

The architecture boundary continues to expose opaque complete-frame extents.
X86-64, AArch64 and RISC-V providers retain their own granules, mapping rules,
boot-memory inputs and ordering obligations. Implementation remains x86-64;
those differences constrain the abstraction and its evidence.

General allocation is a prerequisite for expanding VFS, processes and virtual
memory toward the declared distribution and container workloads. It remains
part of the original implementation goal alongside drivers/Device Tree,
networking/isolation, full retained UAPI and VM hosting.

The subsequent [frame-owner model repair](../../step-4-frame-allocator-readiness.md)
reproduces and corrects same-label authority, identity-exhaustion mutation,
unchecked extent endpoints and malformed normalized-range admission. Its
independent fragmented-allocation oracle and complete semantics suite validate
the existing portable contract. Native general allocation remains required.

[TK-DEC-282](../../step-4-repeated-source-tail-traversal.md) now supplies the
source-plan and bounded native execution stages for repeated ordinary work:
exact natural progress, invariant region transport, carried values, owned
checkpoint/completion and reconstructed source/native/caller publication. Its
initial privileged owner is 64 bytes. The subsequent Count conformance repair
removes a false legacy upper bound from tail primary/isolated source analysis
and observes actual larger extents in guarded native process components. The
[transfer inventory](tail-transfer-readiness.md) identifies actual representation,
authority and resource obligations for actual allocator/metadata integration.
These components do not establish general allocator policy or larger privileged
ownership, which remain required.

The subsequent [ordinary frame policy](../../design/frame-metadata-policy.md)
implements aligned contiguous selection and two-phase whole-extent release over
actual owned metadata cells. Checked helper sharing retains the existing
64-function budget; the policy has 32 retained functions. Its
[component qualification](../../tools/tail-traversal/results/stage-3-frame-policy/qualification.json)
and independent complete-state/native readback pass 137 cases per targeted/full
cohort, including a 72128-byte table, provenance/physical gaps, simultaneous
extents, exhaustion, reuse and retained partial commits. This supplies the
ordinary policy, not the actual privileged allocator: full normalized-map and
reservation binding, physical zero-before-publication, opaque live identities,
dynamic workload requests and owning-caller/context retirement still need kernel
integration. The prior Count/Nat records retain their historical source scope.

The [kernel source obligation](../../design/frame-policy-kernel-binding.md)
now has a distinct root slot, exact allocator-creation binding and immutable
root/library/component replay. Its
[regression record](../../tools/tail-traversal/results/stage-3-frame-source-binding/qualification.json)
and independent readback pass the declaration/context/refusal tests and retained
native cohorts. Native planning and publication explicitly refuse the selected
profile while its actual owning caller remains unqualified. The current old
release lowering still only clears registers; real allocation/release/mapping
lifetimes, physical transitions and dynamic workload requests must be wired to
the new policy before general kernel allocation is qualified.

The subsequent [bootstrap component record](../../tools/tail-traversal/results/stage-3-bootstrap-frame-target/qualification.json)
adds the actual native ownership controller and a privileged x86 bootstrap
target. The constructor's real E820 table contains 152525 allocatable frames
and matches the reference normalization model. Controlled sparse/unaligned
handoffs exercise simultaneous extents, mapping-pin refusal, release/reuse and
physical zeroing before publication; a reserved-zero fault preserves source
writes and pending issuance, poisons the allocator and halts. Eleven cases pass
in each of two cohorts. The real-map case qualifies creation; these allocation
workloads use the controlled handoffs and execute no userspace.

[Independent replay](../../tools/tail-traversal/results/stage-3-bootstrap-frame-target/independent-readback.json)
checks all 174 current/frozen inputs, component templates and identities, native
stack recurrences, actual linked reservations and a separate per-frame map
oracle. The allocation/release component paths are 1840/1824 bytes, excluding
the actual owning gateway. Source-derived identity budgets include the terminal
step. Complete owning-caller/fatal retirement, production root lifetime paths,
dynamic workload requests and canonical publication remain required; the public
gates stay closed. The preceding 163-input source-binding record is historical.

The subsequent [owning bootstrap gateway](../../tools/tail-traversal/results/stage-3-frame-gateway/qualification.json)
qualifies normal/source-abort/post-issuance/late caller retirement in sixteen VM
scenarios, with eleven original-target regression scenarios. It preserves the
actual caller's registers, erases 4032 scratch plus 48 saved-register bytes and
the recovery record, and binds the target to a live invocation lease. Native
dispatch/body/gateway paths use 1952/2024 bytes within the existing 4096-byte
limit. [Independent replay](../../tools/tail-traversal/results/stage-3-frame-gateway/independent-readback.json)
and eight targeted validation commands pass for 178 frozen inputs. This is a
test-built bootstrap component pending production root integration; it does not
qualify syscall stacks, dynamic MM requests or canonical publication. Actual
root allocation/release/mapping lifetimes remain the next implementation work.

The subsequent [full-root record](../../tools/tail-traversal/results/stage-3-frame-root/qualification.json)
passes 362 compiler, 648 language and 253 semantics unit tests and all required
checks for 188 frozen inputs. Its test-built complete boot Root uses actual
source allocations, opaque handles, mapping pins and whole release, including
translation metadata and prepared-process ownership transfers. The full-map
guest observes 67 calls, 14 issued extents and 13 consumed extents; its sole
remaining allocation is the active five-page bootstrap CR3 hierarchy. Independent
complete-map comparison retains all 151977 allocatable frames after the actual
image/resource reservations. An explicitly identified static-ELF ABI fixture
completes; real distribution applications are not qualified by that fixture.

The source now exits completed/failed traversals through an ordinary additional
terminal rule, while retaining full worst-case countdown/counter admission.
[Independent readback](../../tools/tail-traversal/results/stage-3-frame-root/independent-readback.json)
checks source/native/log identities, native stack recurrences, the whole map,
handle/pin/release replay and the linked reservation. This supersedes earlier
source snapshots. The public frame-policy gates remain closed: actual dynamic
workload allocation, syscall-stack binding and canonical publication are still
required. Full persistent MM/VFS/process state and original application,
distribution, container and VM-host acceptance remain open.

The subsequent [runtime-memory record](../../tools/tail-traversal/results/stage-3-frame-memory/qualification.json)
connects actual captured `brk` requests and whole exec/exit heap retirement to
the source allocator. Its fixture executes 80 memory operations, including
3/67/519-page requests, shrink/regrowth, preserved bytes, zero new pages,
within-page changes and invalid requests. Nine context refusals and two fatal
cases check admission and performed-effect retention. A real 50 MiB VM retains
its complete 970-frame topology and correctly refuses a 519-page allocation
when its largest free extent is 384 pages, preserving the old heap and counter.

All 363 compiler, 648 language and 253 semantics unit tests and required checks
pass. [Independent readback](../../tools/tail-traversal/results/stage-3-frame-memory/independent-readback.json)
checks 201 frozen sources, the complete 772-input producer closure, source/native
identities, the 2160-byte combined syscall stack bound and a separate replay of
the small VM's complete metadata/registry state. Peak usage is 1795899392 bytes,
with no limit/OOM events. This supersedes the earlier source scopes. The route
remains test-built: general virtual memory, legacy user-copy integration,
canonical publication and original application/container/VM-host acceptance
remain required; Stage 3 is unfinished.

The [fresh reconstruction record](../../tools/tail-traversal/results/stage-3-frame-reconstruction/qualification.json)
subsequently rebuilds the actual Root/frame source and all ten other selected
libraries into twenty native objects. It binds 804 relocated contributions and
complete allocated/load coverage, deriving rather than trusting the stack,
reservation and linker geometry. The actual image executes the same 80 memory
operations and twelve syscall scenarios. Its complete small-VM map has 972
allocatable frames and a 386-page largest free extent at genuine exhaustion.

All 365 compiler, 648 language and 253 semantics units, four coverage tests and
checks pass for 777 frozen complete inputs. [Independent readback](../../tools/tail-traversal/results/stage-3-frame-reconstruction/independent-readback.json)
checks source/native identities, ELF coverage and an independent complete-state
replay. The combined syscall bound is 2112 bytes; final peak is 2468528128 bytes
with zero limit/OOM events. Program-only publication and public frame boot remain
refused pending complete artifact/provenance/boot coupling. Production-private
reconstruction helpers are not yet connected to that route; workspace checking
reports their unused-code warnings. General MM and original application
acceptance remain required, and Stage 3 remains unfinished.

The subsequent [public artifact qualification](../../tools/tail-traversal/results/stage-3-frame-artifact/qualification.json)
completes Stage 3's source-bearing allocator binding. The compiler pipeline and
boot adapter now rebuild the retained root/frame/all-other-library snapshots,
bind all twenty objects and 804 contributions, and check every source/module,
original producer, caller/load and artifact association. Legacy library metadata
is explicitly compared with those actual source snapshots. Program-only frame
selection remains refused; existing profiles retain their routes.

All 367 compiler, 648 language and 253 semantics unit tests, 261 CLI tests,
four coverage tests and required checks pass for 780 frozen inputs. The actual
public boot image passes 80 memory operations and twelve syscall scenarios;
fifteen atomic publication/boot refusals, null rejection and thirty other-source
identity mutations pass. The 2112-byte combined bound remains within 4096 bytes.
[Independent readback](../../tools/tail-traversal/results/stage-3-frame-artifact/independent-readback.json)
checks all source/native/output identities, ELF coverage, boot payload/resource
association and complete small-map replay (972 frames, maximum free span 386).
Its first failure was a reader-only field mismatch between the public memory
observer and the separately captured Root observer; the corrected reader checks
their common operation/request prefix before replay. Original failed evidence
is retained. Final peak is 2511380480 bytes (2.34 GiB), with zero limit/OOM events
and zero validation swap under the unchanged wrapper and host reserve.

The actual production root still needs to select this route and qualify exact
committed kernel and real application inputs in Stage 4. General MM, legacy
user-copy replacement, persistent VFS/process state and original distribution,
container and VM-host acceptance remain unfinished. These new guests execute
the identified static-ELF ABI fixture, not an unchanged BusyBox executable.
