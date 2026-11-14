# Step 4 implementation status

Step 4 remains in progress. The Topal kernel boots and executes bounded static
userspace under pinned x86-64 QEMU. It does not yet replace the reference VM's
Linux kernel for distribution, Docker/Podman or VM-host qualification.

The approved implementation target remains x86-64. Other architectures inform
systems abstractions; ARM64 implementation is outside the current focus.

The [inactive table-data component](step-4-inactive-image-tables.md) now derives
draft x86 bank geometry and emits an initializer exercised as ordinary user
data. Seven static cases and one VM case pass; four program executions complete
16 valid and 92 refused requests. All nine validation commands and 354 host
models pass over 773 stable inputs. The running kernel and original-profile
rebuild retain the preceding image identity. Actual table ownership, effective
image protection, complete source integration and production qualification
remain unfinished.

The [closed private image stage](step-4-closed-private-image.md) reuses
canonical static/local compilation and source retention for the experimental
process-MM producer. It reconstructs all 23 objects and 862 allocated
contributions while preserving original audit inputs. All 51 unchanged pinned
workloads, four child-exec generations, sixteen ordinary failed execs, selected
compiler/static checks and 354 host models pass over 766 stable inputs.
Independent saved-record readers pass. Mapping builders that enforce image
protection, complete root-backing integration and production qualification
remain unfinished.

The [compiler image-layout component](step-4-compiler-image-layout.md) now derives
image roles and additional leaf storage after complete fresh source/native/link
reconstruction. Five Rust tests, workspace/static checks and 354 host models
pass over 764 stable inputs. An unowned GOT entry remains refused. This is an
opt-in descriptive compiler product; complete native backing, mapping builders,
effective protection and production integration remain unfinished.

The preceding [kernel-image protection design/model stage](step-4-kernel-image-protection.md)
adds an explicit role/ownership contract across all usable roots and aliases,
preserving bootstrap-equivalent meaning and core semantics. Twelve new model
tests include all 256 small role/hole patterns, current x86 subdivision and
larger-granule conflicts. All 353 host models and exact frozen replay pass.
This creates no live authority, native mapping, protected context or public
activation. The current kernel and its hardware-permission gaps are unchanged.

The preceding [calendar placement stage](tools/user-memory/results/stage-3-calendar-placement.md)
moves 4,864 Gregorian data bytes out of the process instruction body into
read-only/non-executable ELF data loads. Both fresh native builds produce the
same new kernel; all 51 pinned workloads, four child-exec generations, sixteen
ordinary rejected execs, selected compiler checks and 341 models pass. All 759
watched inputs remain stable. Frozen replay preserves the full native records.
Fresh listings agree on all 848 selected privileged controls. Saved mappings
still have writable supervisor routes and clear execute-disable bits where
resolved; 32 endpoint checks at final retirement remain unresolved. Hardware
protection, complete entries/lifetime and public activation remain unqualified.

The preceding [calendar-data audit](tools/user-memory/results/stage-3-calendar-data.md)
identifies the selected decoder discrepancy as the low byte of a Gregorian
day-count table entry. Exact table/consumer bytes, source reconstruction and
conditional graph checks pass, together with 337 host tests and frozen replay.
The differing linear listings remain retained. This does not prove complete
entry exclusion, hardware NX, runtime immutability or production qualification.

The [rejected child-exec cohort](tools/process-memory/results/stage-3-exec-failures.md)
adds sixteen ordinary failed execs with retained child/parent state, followed
by four successful replacements. Close-on-exec flags remain local to the child,
survive rejection and close its descriptor only on success. Native checks,
independent readback, eight new controls, 330 models and frozen replay pass.
The kernel and boot image are unchanged; broad acceptance remains unfinished.

The [ordinary child-exec signal cohort](tools/process-memory/results/stage-3-exec-signals.md)
now checks fork inheritance, child-only changes, successful-exec action reset,
blocked-mask persistence and parent restoration in four native generations.
The kernel and boot image are unchanged. Independent saved-record checks and
frozen replay preserve the initial reader failure; signal delivery, production
activation and the original distribution/container/VM-host target remain open.

The experimental [owned user-memory work](step-4-owned-user-extents.md) now
includes [selected clock/random stack retirement](tools/user-memory/results/stage-3-stack-retirement/README.md).
Thirteen normal-return cleanup blocks preserve registers/flags and the existing
stack bounds, with fresh native regressions and exact emission comparison.
The subsequent [ordinary metadata cohort](tools/user-memory/results/stage-3-source-metadata/README.md)
checks complete filesystem/descriptor transitions and observes the remaining
clock callers, giving all thirteen selected cleanup blocks in same-kernel union.
The subsequent [owner-boundary cohort](tools/user-memory/results/stage-3-owner-boundaries/README.md)
checks 104 actual allocation/translation snapshots around 52 stat/write helper
calls. Boundary equality does not establish a continuous mapping lifetime.
The [subsequent interval observation](tools/user-memory/results/stage-3-owner-intervals/README.md)
guards 26 selected identity-address ranges through each of those 52 copies.
It records twelve ordinary write controls and preserves the two-CPU VM; this
does not qualify physical aliases, DMA or all asynchronous behavior.
The subsequent [recorded mapping analysis](tools/user-memory/results/stage-3-mapping-aliases/README.md)
checks complete paging graphs at 156 observed boundaries, all 572 selected user
pages and unique supervisor identity routes to the 26 protected ranges.
Independent direct-table readback, 164 models and twenty offline controls pass.
There is no new native execution; stale TLB, DMA and continuous mode/asynchronous
exclusion remain required.
The subsequent [local heap-publication checkpoint](tools/user-memory/results/stage-3-translation-publication/README.md)
observes 20 publications across 36 requests, all 812 invalidation-loop states,
sixteen old-backing releases and final detached-root cleanup. Native observation
binds the flushing CR3 handoff; it does not treat bootstrap-context invalidations
as user-root invalidations. The complete ordinary VM, 177 models and eighteen
offline controls pass. Other mapping editors and general stable lifetime remain
unqualified.
The [linked paging-control inventory](tools/user-memory/results/stage-3-paging-controls/README.md)
retains 847 selected sites, ten control-register writes and 29 invalidation sites
across two full executable-section listings. Nine targeted tests and exact
frozen replay pass. The subsequent calendar audit explains the selected decoder
discrepancy's data context. Other unknown/overlapping rows and the embedded opcode
candidate retain entry questions; no new guest or mode proof runs.
The subsequent [image-publication checkpoint](tools/user-memory/results/stage-3-image-publication/README.md)
observes four RELRO edits, three exec restorations and all 86 image invalidation
states, retaining the previous heap/owner/metadata checks. The complete ordinary
VM, 196 models, sixteen image controls and eighteen heap controls pass. Remaining
editors, continuous mode/lifetime and production gates remain required.
Actual ownership/lifetime and complete caller qualification, canonical production
selection and original kernel acceptance remain required.

The [process-MM foundation](step-4-process-memory-ownership.md) addresses the
remaining fixed child/pair heap snapshot and shared active MM state. Its design
preserves qualified child/pipe behavior with independent frame-backed heaps.
Seventeen portable tests, 3,200 application-oracle transitions and saved-data
controls pass. This portable stage changes no native behavior; actual integration,
mapping-producer migration and public production qualification remain required.
Its [first native preparation component](step-4-process-memory-ownership.md#stage-2-first-native-preparationcancellation-component)
now copies, pins, cancels, erases and releases actual heaps in 32 ordinary `brk`
invocations, including four 67-page heaps. The complete VM, 196 existing models,
17 process-memory models, source/metadata readback and fourteen saved-data controls
pass. This does not switch processes or activate the replacement in production;
active/inactive binding, native failure paths and full process integration remain.
The subsequent [independent process/MM component](step-4-process-memory-ownership.md#stage-3-first-independent-processmm-integration)
passes eight real fork/clone creations and exits, eight handoffs, independent
parent/child resize at 67/101 pages, wait-output retry and three parent execs in the
complete ordinary ABI witness. Saved records cover 32 consumed/zeroed extents,
with twenty offline controls and thirteen byte-identical predecessor artifacts.
That archive is a test-only selection: child exec, live-child cancellation, complete
caller/loan/mode/publication/fatal proofs and production reconstruction remain
required. The full original step remains incomplete.
The subsequent [child-exec cohort](step-4-process-memory-ownership.md#child-exec-and-replacement-startup)
passes four child execs while retaining the parent, fresh empty/new child heaps,
actual FS/registration reset and later FS restoration. It also passes the full
ABI witness, checks 101 MM boundaries and 36 consumed/zeroed extents, and rejects
thirty saved-data controls. Imported process ownership now stays usable during
replacement startup independently of the image-stage counter. This remains a
test-only component; live-child cancellation, broader exec/mapping/lifetime/fatal
cases and production qualification remain required.
The subsequent [parent cancellation cohort](step-4-process-memory-ownership.md#parent-termination-with-a-live-inactive-child)
passes both parent exit operations at shell and early replacement startup with
a child parked on a real pipe read. It retains independent parent/child heaps,
writer aliases and a child-only dirty anonymous file, then observes complete
inactive retirement, descriptor/shared-service destruction and detached active
cleanup. Four native cases pass with the existing stack/physical bounds. This
remains an experimental selection; other wait/relationship shapes, complete
native lifetime/fatal proofs, application regressions and public activation
remain required.

The subsequent [blocked-writer cancellation cohort](step-4-process-memory-ownership.md#blocked-writer-cancellation-and-dirty-fifo-retirement)
passes both parent exit operations at shell and early replacement startup with
a child parked on a full FIFO write. It checks all dirty FIFO bytes, inverse
endpoint ownership and whole-store erasure using the byte-identical kernel from
the preceding cancellation stage. Four new VM cases and 64 saved-data controls
pass. This extends experimental cancellation evidence; allocation-miss, other
relationship, continuous lifetime/fatal and production gates remain open.
The subsequent [physical-refusal cohort](step-4-process-memory-ownership.md#physical-allocation-refusal-and-ordinary-retry)
passes real capacity and fragmentation failures in a 52-MiB guest using the
same kernel. Complete firmware/catalog/registry readback binds both failures;
ordinary ENOMEM return preserves descriptor/PID-policy state, followed by
successful one-page and empty-heap forks using PIDs 2 and 3. The native test,
27 process-memory tests, 21 new controls and retained-regression readback pass.
Other resource/identity/fatal, continuous lifetime and public-kernel gates
remain required.

The subsequent [maximum-copy cohort](step-4-process-memory-ownership.md#successful-maximum-copy-and-emptyone-page-handoff)
passes successful ordinary fork and fork-like clone with complete independent
519-page heap copies, followed by four zero/one-page child handoffs and retained
518-page parent contents in each case. Both native runs, 44 saved-data controls,
complete erasure readback and current readers for retained same-kernel
regressions pass. General process/MM geometry, all callers/loans, identity/fatal
and continuous lifetime/public-activation gates remain open.

The subsequent [runnable/zombie cohort](step-4-process-memory-ownership.md#runnable-and-zombie-child-relationships-at-parent-termination)
passes eight parent-termination cases across both exit operations and startup
positions. Runnable children retain their dirty heap/file until cancellation;
zombies retain unreaped status after their resources retire, with no duplicate
retirement during parent exit. All 176 saved-data controls and current readers
for twelve retained same-kernel native runs pass. This adds bounded lifetime
evidence; general orphan/init behavior, broader caller/loan/fatal and production
qualification remain open.

The [native ELF audit reader](step-4-process-memory-ownership.md#elf-symbol-scopes-in-native-audits)
now preserves distinct local symbols in complete address-based enumeration and
rejects ambiguous named lookups. Seven scope controls bring the user-memory
suite to 203 passing tests. This prepares ordinary-interface process-MM
regressions; it does not itself qualify new VM behavior or production use.

The subsequent [ordinary-interface matrix](step-4-process-memory-ownership.md#ordinary-application-interfaces-on-the-process-mm-kernel)
passes all nine existing source-built cohorts on the experimental process-MM
kernel: transfers, exec refusals, signals, poll, pipes, storage, metadata, serial
and stat. It records 36 source invocations, 2,032 application-interface syscalls,
702 MM boundaries and 354 retired extents, with 139 saved-data controls.
Whole MM histories and selected interface reconstruction pass, alongside fresh
203 user-memory and 31 process-memory tests. Twenty previous native histories
remain identified on the byte-identical kernel. An initial uninstrumented
metadata observer assertion remains unexplained despite two later successful
runs. Broader caller/loan/fatal, continuous lifetime, unchanged application-corpus
and public-activation gates remain required; initial Step 4 is incomplete.

The subsequent [unchanged application corpus](step-4-process-memory-ownership.md#unchanged-application-corpus-on-the-process-mm-kernel)
passes all **51 pinned BusyBox workloads** on the same experimental kernel.
It records **1,054 MM boundaries**, **333 retired extents / 25,694,208 erased
backing bytes**, and **859 saved-data controls**. Complete final catalog/registry
reconstruction binds all 152,008 firmware frames per VM. First-reach samples
cover 66 of 91 extent-helper sites, without every-path or lifetime qualification.
Fresh 203 user-memory and 37 process-memory tests pass. Nine interface and
twenty process histories remain identified as previous native executions on
the byte-identical kernel. Remaining caller/loan/mapping, identity-width/fatal,
continuous lifetime and public activation work still blocks initial Step 4 and
the original full Linux/distribution/container/VM-host objective.

The subsequent [process-publication matrix](step-4-process-memory-ownership.md#process-publication-and-retirement-observations)
passes all seven selected tests / **21 VMs** and complete source-bound readers.
It records **279 publications**, **26,191 invalidation states**,
**45 equal-size process publications** and **166 retired
extents / 46,915,584 erased bytes**, with 252 new saved-data controls.
The kernel remains byte-identical to the application-corpus kernel. Fresh
workspace/all-target and 203/59 model checks pass; seven LLVM modules verify per
build and thirteen predecessor contributions remain unchanged. Frozen replay
repeats the new records and identified previous 51 application, nine interface
and twenty process histories. Prepared-token cancellation, every caller/loan/
mapping writer, continuous/AP/DMA/asynchronous lifetime, fatal/identity/resource
and cold public reconstruction remain open. This completes the local observation
component, not initial Step 4 or the complete Linux/distribution/container/VM-host
objective.

The subsequent [source-built failure diagnostics](step-4-process-memory-ownership.md#source-built-process-policy-failure-diagnostics)
pass one fresh ordinary process VM and two diagnostic VMs. They check retained
performed effects, held child/parent/candidate identities, poison and declared
4,000-byte direct-MM caller retirement before and after replacement publication.
All 27 diagnostic altered-record controls reject; frozen saved-record replay,
fresh compiler/workspace/model checks and seven LLVM modules per build pass.
The broader suite's original temporary-filesystem quota failure is retained;
fresh qualification uses task-owned disk scratch with unchanged host protections.
The ordinary kernel remains byte-identical; deliberately incorrect diagnostic
policies have separate source/kernel identities. These selected failures do not
qualify every fatal path, the outer caller, continuous lifetime or public
activation. Initial Step 4 and the original full Linux objective remain open.

The subsequent [partial-source failure cohort](step-4-process-memory-ownership.md#partial-source-status-failures-after-performed-stores)
passes five fresh VMs: the ordinary cohort, both completed-invalid-delta
diagnostics and two source-status failures after 13 stores. Actual source-result
decoding and native fatal call sites bind retained metadata/issuance, every held
heap identity, poison and declared 4,000-byte direct-MM invocation retirement.
All 70 diagnostic saved-record controls, five-reader frozen replay, six fresh
validation commands and seven LLVM modules per build pass. A guarded reference
to the earlier full 383-test compiler suite is explicit; that suite is not newly
rerun for this private-test/observer increment. The older three kernel identities
remain unchanged. Other fatal paths, caller/loan/mapping lifetimes, asynchronous
access and public/full Linux qualification remain open.

The subsequent [combined caller census](step-4-process-memory-ownership.md#combined-first-reach-caller-census)
reconstructs first-reach evidence for 88 of 91 helper sites from the earlier
51 application and nine interface runs. Two old interface records lacking
first-reach fields are explicitly excluded. All ten census models, frozen
saved-data/native-byte replay and 73 surrounding process-memory tests pass.
This runs no new VM or compiler build. The three unsampled legacy branches
remain explicit gaps; samples do not establish admission, every path, private
loans or continuous lifetime. Step 4 and public activation remain unfinished.

The subsequent [pre-import missing-stat prefix](step-4-process-memory-ownership.md#native-pre-import-missing-stat-caller)
reaches helper 101,781 through an ordinary source-built syscall and observes
successful admission/return and all 4,080 actual caller bytes retired. A separate
fresh ordinary process/MM VM completes. Both native runs and frozen saved-data
readers pass, alongside a fresh full 383-test compiler suite, 203 user-memory
models and 74 process-memory models. The same-kernel union now samples 89/91
helper sites; offsets 88,215 and 94,687 remain unsampled. The prefix itself stops
at its userspace marker. Other caller/loan/fatal/continuous-lifetime, production
and full initial Linux acceptance remain required.

The subsequent [ordinary caller-retirement cohort](step-4-process-memory-ownership.md#normal-return-caller-retirement-across-ordinary-processmm)
observes 75 distinct normal returns, with 72 actual 80-byte and three 88-byte
outer adapters, complete caller/private erasure and retained whole MM bytes.
All three native VMs pass; the final fresh qualification and frozen replay pass
203 user-memory and 80 process-memory models. Two earlier reader/test-discovery
failures are preserved and reproduced. The preceding full 383-test compiler
suite is referenced over unchanged inputs, not rerun. The kernel, C workload
and application bytes remain unchanged. With the prior stat prefix, 76 sites
have selected normal-return evidence; historical first-reach remains 89/91.
Every-path, continuous/private-loan/fatal, production and initial Linux acceptance
remain unfinished.

The subsequent [interface caller-return cohort](step-4-process-memory-ownership.md#normal-return-callers-in-ordinary-interface-cohorts)
passes the existing poll, pipe, storage and serial VMs and adds thirteen sites,
bringing selected normal-return evidence to **89/91**. Both actual outer adapter
alignments and complete caller/private erasure are observed. The original new
reader's null-field failure is retained and reproduced; its corrected replay
and six supplementary checks pass, including 203 user-memory and **82**
process-memory models, without a VM rerun. Kernel/application bytes and the
original interface checks remain unchanged. Two review comparison errors are
also retained. Offsets 88,215 and 94,687, every-path/lifetime/fatal/publication,
production and full initial Linux requirements remain open.

The subsequent [conditional callee graph](step-4-owned-user-extents.md#stage-3-conditional-linked-callee-graph)
checks 71,457 linked instructions, six registered callback sites and six guarded
jump tables with 98 entries. Its selected edge set closes under explicit
entry/stability/call-return assumptions, with `qualification_complete=false`.
All 219 user-memory and 82 process-memory models and eleven altered native-record
controls pass; frozen replay preserves the graph and reproduces the retained
recovery-reader failure. No VM or kernel rebuild runs. Helpers 88,215/94,687,
global entry/lifetime/effect/return proofs, production and initial Linux acceptance
remain unfinished; native selected return coverage stays 89/91.

The subsequent [saved boot/IDT binding](step-4-owned-user-extents.md#saved-boot-and-initial-interrupt-entry-binding)
checks the private packaged image against its ELF loads, BSS, transition,
initial descriptors and tables. Five additional bound entries expand the
conditional graph to 206,533 instructions and 995 function identities. Thirteen
offline image controls, 229 user-memory models, 82 process-memory models and
exact frozen replay pass. It also finds an advertised independent 64-bit boot
entry pointing into zero-filled payload, which requires an adapter correction.
This adds no native execution or production provenance. Actual loader/CPU state,
complete asynchronous effects, runtime entries/lifetime and the original Linux
acceptance remain open; the 89/91 native helper coverage is unchanged.

The subsequent [boot-header correction](step-4-owned-user-extents.md#boot-header-advertisement-correction)
clears the unimplemented independent 64-bit entry advertisement in adapter
revision 154. A fresh source-built process-MM Q35 regression passes; its kernel
is byte-identical and its complete image changes only at header byte `0x236`.
Two targeted compiler tests, 230 user-memory tests, 82 process-memory tests and
the surrounding checks pass, with exact saved-data replay. The initial full
compiler run was intentionally stopped after identifying legacy debugger-injected
kernel-state cases and remains incomplete. The corrected native component is
unpublished; separate loader, full fatal/entry/lifetime/production and original
Linux acceptance obligations remain open.

The subsequent [saved gateway model](step-4-owned-user-extents.md#saved-pre-invocation-gateway-checks-and-retained-diagnostics)
checks 20,504 cases over four compiled gateways against an independent selected
pre-invocation predicate. Thirty-six weakened-guard controls and 248/82 model
tests pass, with exact frozen replay. Retained files also establish completion
of 36 predecessor diagnostic cases before the earlier stop, superseding the
previously unknown reach; the full compiler command remains interrupted. This
is fresh host interpretation and read-only inventory, with no new VM execution,
legacy-test removal, owner authority or production activation. Other operations,
real entry/caller/owner/lifetime and full original Linux acceptance remain open.

The [repeated-source milestone](step-4-repeated-source-tail-traversal.md) now has
a source-only tail-plan inspector and an ordinary byte-traversal source.
All 636 frontend and 295 compiler tests pass. Carried-value/storage-reuse proofs,
native execution and real allocator integration remain its subsequent stages;
this does not add an executed kernel capability.

## Current executed capabilities

Shared regular names now support hard links and rename/replacement/exchange
through actual ordinary source loans. The
[current qualification](step-4-shared-namespace-transactions.md) passes 2349 Rust
tests, 230 Topal VM observations, 51 unchanged BusyBox workloads and 45 lab
publication contracts. All four chmod-family calls retain shared inode behavior
under V6 canonical metadata admission. This covers the prepared root and bounded
object kinds; general credentials and filesystems remain required below.

The subsequent [frame-owner model repair](step-4-frame-allocator-readiness.md)
passes all 239 semantics tests and locked workspace all-target checking. It
strengthens the reference allocator's authority, refusal and range checks;
general native allocation remains required.

The [systems implementation maintenance](step-4-modular-systems-implementation.md)
restores the existing size ceiling across all 381 owned Rust sources. Complete
workspace checks, preserved function/assertion inventories and an independently
compared fresh primary VM pass. This changes no kernel capability or normative
contract; the preceding 137 VM observations remain separately retained evidence.

- Independently bound ordinary region instances, with exact caller-derived
  identities, separate private code/data/hooks and disjoint temporary storage.
  Root zero-fact Boolean and file-position two-fact callbacks coexist under one
  frame ABI; false root completion clears temporary storage before refusing
  owner resumption. Legacy contexts remain separately selectable. See the
  [instance report](step-4-instance-scoped-region-actions.md).

- Bounded framed inputs for ordinary region functions: exact caller-declared
  zero-to-sixteen Nat facts, full-u64 transport, complete immutable input-frame
  ownership/lifetime and stack/allocation evidence. The existing file-position
  caller supplies its two facts through owned staging, with complete cleanup
  and a separately selected legacy-ABI execution control. See the
  [framed-action report](step-4-framed-region-actions.md).

- Complete allocated code/read-only sections for every selected finite/numeric
  policy slot, including relocated tables, with exact object/manifest/linked
  bindings and independent changed-byte boot refusal. Source/native interfaces,
  resource certificates and caller/cleanup contracts retain their meanings.

- Complete immutable Unit/action region inputs bound independently of code,
  with exact object-to-linked comparison and changed-literal boot refusal;
  Unit entry ABI and empty-data functions retain their contracts.

- Ordinary Topal arithmetic and imported byte codecs update actual persistent
  shared file-position fields under an exclusive controller grant. Real syscall
  integration supplies effective base/count, preserves append/alias/fork/exec
  behavior and proves temporary cleanup/history retirement; positional/zero/EOF/
  error paths issue no action.

- Ordinary effectful Topal byte helpers on actual root-owned storage, suspended
  owner access, dynamic output guards and complete arena/control/retired-stack
  erasure before owner resumption, with physical debugger/refusal evidence.

- Positional pread64/pwrite64 over sparse and immutable files, invocation-owned
  offsets, Linux append placement and unchanged shared description positions.
- Named RAM files with creation/exclusive/truncating opens, independent positions,
  unlink-open lifetime, exact long directory records and complete reclamation.
- Versioned ordinary Topal file-open disposition for existing and missing targets,
  named/anonymous creation and truncation, with frozen compatibility.
- An ordinary Topal Boolean/Enum policy library with source-bound code ownership,
  actual helper calls, native ABI and stack evidence, and representation adaptation.
- Ordinary Topal numeric deadline policy using shared arbitrary-precision algorithms,
  exclusive owned arena lending, allocation/stack proofs and complete temporary erasure.
- Closed ordinary source-module snapshots for the selected policy interfaces, including
  transitive helpers, real module debug locations and digested dependency bytes.
- Nested immutable numeric Records with named fields/reconstruction, complete
  aggregate stack/allocation bounds and an imported structured deadline policy.
- Bounded immutable integer Arrays with checked Optional reads, exact per-entry
  allocation bounds and imported collection helpers using shared ordinary runtime.
- Source ELF64 header interpretation over copied owned bytes, precommit exec
  refusal, independently selected header/time policies and complete reclamation.
- Versioned ordinary source selection of 70 Linux operations plus Unsupported,
  with frozen 55/57/59/61/65-operation compatibility, actual nominal tag adaptation and
  cleanup before native argument/handler access.
- Source process-ID/cursor transitions over immutable Record snapshots, named
  scalar result extraction and owned state publication after complete cleanup.
- Actual ordinary identity-state Records retained between callbacks, with
  published graph borrowing, independent successor preparation, cleanup before
  publication, last-borrow retirement and complete store reclamation.
- Complete source-produced nested state with Boolean history, full 64-bit
  creation counts, ordinary nested presentation order and shared checked layout/
  projection/native validation over independently owned successor graphs.
- Generated privileged entry/return, checked memory/translation ownership,
  kernel contexts, notifications, clock/deadline preemption and complete rollback.
- A supplied static ELF in the qualified mapping profile, real user execution,
  caller-owned argv/environment, repeated exec and normal exit status.
- One independently owned child with private memory/registers/TLS, shared inherited
  descriptions, child exec/exit, restored parent and once-only native wait.
- Cooperative handoff between both live processes, native WNOHANG/blocking child
  wait, independent saved continuations and complete active/inactive teardown.
- Shared anonymous pipes with checked atomic/partial transfers, EOF, exact
  read/write readiness, native metadata and blocked-owner cancellation.
- Shared file descriptions and offsets, descriptor flags/aliases, an immutable
  boot-root namespace, cwd/directory observations and prepared root/PID/UTS views.
- Owned sparse RAM-file storage in `/tmp`: four named/anonymous regular
  objects over eight shared pages, actual read/write/seek/truncate,
  coherent directory/file metadata, private umask, alias/fork/exec sharing and
  complete last-reference and active/inactive termination reclamation.
- Signal registration/masks without user delivery; actual RTC seconds; borrowed
  monotonic time/resolution and relative sleeps with source-defined saturation.
- Unmodified pinned BusyBox utilities and interactive shell builtin/cwd/input
  cycles, file redirection/append/reopen/list/remove and command substitution through
  shared services, including transfers beyond FIFO capacity. Unknown syscall
  selectors return ENOSYS.

The [current runtime report](step-4-positional-file-transfers.md) records supplied
offsets, append/error ordering, exact native positions and bytes, all four source
dispatch versions, active/inactive cleanup and independent model/native/Linux/VM
evidence. The [named-file report](step-4-named-volatile-files.md),
[source-plan report](step-4-source-file-open-plans.md) and
[sparse-file report](step-4-owned-volatile-files.md) retain their earlier qualified
identities and foundations. Earlier interfaces remain explicit compatibility
controls. Each report identifies its scope and limitations. A successful bounded
witness does not establish general Linux compatibility.

## Remaining acceptance work

The [shared namespace increment](step-4-shared-namespace-transactions.md)
qualifies independently owned entries/inodes, ordinary source actions, all five
V6 adapters and changed table consumers. Its 47 namespace VM observations cover
refusals, source faults, clock failures, lifecycle, physical cleanup and unchanged
mv/ln workloads. The complete 230-observation refresh and all 45 publication
contracts pass; full initial acceptance remains below.

The [reusable-page action stage](step-4-source-ram-page-allocation.md) was qualified
at the TK278 checkpoint:
ordinary Topal updates the real filesystem allocation map, and the controller
validates exact deltas before association publication. Its portable model,
independent first/last-fit source components, actual x86-64 owner binding and
21 targeted VM dispositions pass, alongside 137 freshly repeated preceding
VM observations and all 42 lab contracts.

After the user freed host RAM, complete Rust qualification passed under the
unchanged memory policy: all-target workspace checking, all 2261 tests across
41 targets, and the separate doc-test command. The
[exact record and logs](tools/rust-qualification/results/tk278/final/qualification.json)
bind the qualified revision and tool identities. Formatting and the 2000-line
Rust ceiling also pass. Qualification corrected three stale controls: a legacy
fixture selecting the new page declaration, the systems-rule count, and the
provider-object relocation/import expectations. The latter now checks both
allocator profiles and proves that its forward descriptor reference has a
matching strong local BSS definition. The earlier failed attempt remains
archived. No production code or qualified kernel changed in those corrections.
The optional hosted workflow stays inactive; local qualification needs no
workflow-location exception.

This adds a bounded reusable RAM-page owner. General allocation and the initial
distribution/container/VM-hosting acceptance remain unfinished.

The next [unsigned-field stage](step-4-unsigned-region-fields.md) defines explicit
portable encodings on existing borrowed regions. Its actual-owner model and
all 190 semantics tests pass, with nine new encoding/refusal/lifetime tests.
Typed source, exact runtime/cost binding and independent native components now
pass: all sixteen formats, 92622 C calls and both byte-order GDB observations
over actual source values, payload access and complete temporary erasure.
The current file-position codec now uses the checked eight-byte store: its
complete temporary bound is 464 bytes and its combined owning stack is 712.
All eight source/native/binding tests and thirteen fresh targeted VM observations
pass, including actual field/primitive/cleanup evidence and frozen bytewise
interface controls. Complete locked Rust qualification now passes all 2280 tests
across 41 targets, all-target checking and the separate doc-test command, with
the [exact record/logs](tools/rust-qualification/results/tk279/final/qualification.json)
archived. Its complete affected machine cohort now passes **158 fresh VM
observations**, including forty unmodified BusyBox workloads; all **42 lab
publication contracts** pass. The existing source arena and core language
meaning are unchanged. This qualifies the unsigned-field increment; the initial
acceptance areas below remain unfinished.

The current [shared-inode mode stage](step-4-shared-inode-modes.md) executes all
four chmod-family calls through an ordinary source action on actual regular,
directory and pipe metadata. Its first two commits define the shared-state and
actual-loan models and qualify 36906 independent native component calls. The
third stage selects 65 operations, binds canonical inode geometry and complete
12208/1296/1544-byte temporary/source-stack/combined-stack bounds, passes 33
pinned Linux comparisons, twelve physical VM dispositions and five unchanged
BusyBox chmod/stat/shell workloads. Complete owner-byte deltas, actual Nat facts,
registers, frame/arena/retired-stack reclamation, generation lifetime,
inactive-child cleanup and pre-entry/post-store refusal controls pass. All 264
compiler unit tests, 628 language tests, 209 semantics tests, locked all-target
checking and four specification/requirement coverage checks pass. The complete
committed Rust rerun passes all **2305 tests across 41 targets**, locked all-target
checking and the separate doc-test command, with the
[exact record/logs](tools/rust-qualification/results/tk280/final/qualification.json)
archived. The complete affected machine cohort passes **176 Topal VM
observations**, including **45 unchanged BusyBox workloads**; all **43 lab
publication contracts** pass. Its [final manifest/logs](labs/qemu/x86_64/results/tk280/final/qualification.json)
retain current record hashes and exact counts, separately preserving failed
attempts. Host safeguards and core language meaning are unchanged. This qualifies
the inode-mode increment; preceding-kernel evidence remains historical and the
original acceptance areas below are unfinished.

Ordinary effectful functions now execute over actual
[borrowed kernel regions](design/borrowed-kernel-regions.md). The
[model foundation](step-4-borrowed-region-model.md) and
[source/object stage](step-4-source-region-functions.md) lead to the
[root invocation report](step-4-root-region-invocation.md): actual owner bytes,
ordinary helper frames, dedicated root hooks and physical cleanup. The
[persistent-field report](step-4-persistent-region-actions.md) adds a real
subsystem operation: ordinary source arithmetic and a checked codec on the
actual shared eight-byte position, with repeated invocation/terminal cleanup.
General persistent allocation, broader ownership and kernel subsystems still
need implementation and qualification.

The [framed-input refinement](step-4-framed-region-actions.md) removes the fixed
two-scalar transport restriction for qualified source objects while retaining
exact per-caller signatures. It changes no Linux operation or core-language
meaning and does not supply general persistent allocation or subsystem coverage.

The [instance refinement](step-4-instance-scoped-region-actions.md) removes the
single-object naming restriction and binds each qualified caller independently.
It retains the ordinary signature/frame transport and first integrates root and
field owners; general persistent allocation and broader callers remain required.

The [allocator and storage readiness inventory](knowledge/common/allocator-readiness.md)
connects the approved frame/region ownership and ordinary recursion contracts
to current compiler restrictions. General allocation still requires workload-
derived extents, proven repeated source execution and complete temporary and
persistent lifetime evidence.

| Area | Required result | Current limit |
| --- | --- | --- |
| Freestanding Topal libraries | Kernel policy/subsystems in ordinary Topal modules over qualified machine elements, as approved in Step 2 | Ordinary byte helpers with actual root loans and persistent file-position actions, explicit controller exclusion, complete code/data binding and cleanup; finite Boolean/Enum and bounded numeric policy with closed imports, immutable Records/Arrays, checked Optional reads, copied header interpretation, nominal syscall selection and source-produced nested retained identity state; broader types, general persistent allocation and subsystem effects remain; native adapters still carry substantial Linux policy |
| Processes and execution | Process/thread creation, wait, general executable/mapping selection, dynamic ELF and libraries | Initial parent and one additional resumable child; one pinned static executable/mapping profile |
| Signals and scheduling | Delivery, interruption/restart, multiple runnable users, lifecycle and synchronization | Registration/masks, private thread registrations, kernel-context foundations, cooperative pair handoff and child/FIFO waits; no general user preemption, fairness or signal delivery |
| Filesystems/devices | Writable storage, root mount, general VFS and actual distribution device interfaces | Immutable boot files, `/tmp` and four named/anonymous sparse RAM files over eight shared pages; linked-file create/reopen/unlink, positional transfers and dynamic directory records; hard links and rename/replacement/exchange over independent entry slots; no persistent storage or new directories; four shared FIFOs and qualified serial/entropy/time hardware; broader pipe controls and nonignored SIGPIPE remain |
| Board descriptions and Device Tree | Interoperate with the DTS-to-DTB toolchain, validate DTB inputs and selected board bindings with provenance (`LK-DT-001`–`004`) | [Research and binding inventory](knowledge/common/device-tree.md); pinned Q35 platform and architecture-provider qualification only; DTB consumption and binding-driven device construction remain |
| Networking and isolation | Network stack, namespaces, credentials/capabilities, cgroups and other retained UAPI contracts | Research/design plus bounded initial-process views |
| Reference distribution | Replace the Step 3 kernel and run declared command-line corpus from its ordinary prompt | Toolchain-gate VM and static BusyBox corpus |
| Containers | Docker and Podman acceptance in the replacement-kernel VM | Reference Linux controls only |
| VM hosting | Declared guest VM operation through Linux-compatible virtualization interfaces | Guest execution under QEMU only |
| Full pinned UAPI | Complete retained interface inventory and regression/drift evidence | Explicitly qualified subsets; ENOSYS does not imply implementation |

These are existing initial-plan/Step 2 obligations, not new scope. See the
[Step 2 architecture](design/topal-kernel-architecture.md),
[gap analysis](design/topal-gap-analysis.md), [baseline](baseline.md) and
[Step 3 reference VM](step-3-report.md). Systems-only refinements proceed under
standing authorization; a necessary core-language design change still requires
a human decision. Human merge remains required, without pausing implementation
merely because a PR is open.

## Review packaging

The open Step 4 work is consolidated into one PR against the repository default
branch. Definition, specification, implementation and qualification stages remain
separate commits. Further Step 4 increments update that PR. Previously published
increment PRs retain their historical review/evidence references and are
superseded by the consolidated PR. No automatic merge is authorized.
