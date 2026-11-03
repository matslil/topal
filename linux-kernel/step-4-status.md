# Step 4 implementation status

Step 4 remains in progress. The Topal kernel boots and executes bounded static
userspace under pinned x86-64 QEMU. It does not yet replace the reference VM's
Linux kernel for distribution, Docker/Podman or VM-host qualification.

The approved implementation target remains x86-64. Other architectures inform
systems abstractions; ARM64 implementation is outside the current focus.

## Current executed capabilities

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
- Versioned ordinary source selection of 61 Linux operations plus Unsupported,
  with frozen 55/57/59-operation compatibility, actual nominal tag adaptation and
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

| Area | Required result | Current limit |
| --- | --- | --- |
| Freestanding Topal libraries | Kernel policy/subsystems in ordinary Topal modules over qualified machine elements, as approved in Step 2 | Ordinary byte helpers with actual root loans and persistent file-position actions, explicit controller exclusion, complete code/data binding and cleanup; finite Boolean/Enum and bounded numeric policy with closed imports, immutable Records/Arrays, checked Optional reads, copied header interpretation, nominal syscall selection and source-produced nested retained identity state; broader types, general persistent allocation and subsystem effects remain; native adapters still carry substantial Linux policy |
| Processes and execution | Process/thread creation, wait, general executable/mapping selection, dynamic ELF and libraries | Initial parent and one additional resumable child; one pinned static executable/mapping profile |
| Signals and scheduling | Delivery, interruption/restart, multiple runnable users, lifecycle and synchronization | Registration/masks, private thread registrations, kernel-context foundations, cooperative pair handoff and child/FIFO waits; no general user preemption, fairness or signal delivery |
| Filesystems/devices | Writable storage, root mount, general VFS and actual distribution device interfaces | Immutable boot files, `/tmp` and four named/anonymous sparse RAM files over eight shared pages; linked-file create/reopen/unlink, positional transfers and dynamic directory records; no persistent storage, new directories, rename or hard links; four shared FIFOs and qualified serial/entropy/time hardware; broader pipe controls and nonignored SIGPIPE remain |
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
