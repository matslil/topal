# Owned user-memory extents

`TK-DEC-283`, under the standing approval for systems-only changes. This refines
`TK-ELEMENT-USER-ACCESS-001` and follows [runtime frame memory](dynamic-frame-memory.md).
It changes no core syntax, value meaning, source pointer construction or
exception mechanism. Native realization is initially x86-64; AArch64 and RISC-V
remain abstraction inputs.

## Purpose and boundary

The production frame profile can grow its heap beyond the predecessor's 34
pages. Its shared extent helper and several independent syscall adapters still
limit buffers to the predecessor geometry. Some helpers inspect only leaf
permissions. Removing one bound would leave both incomplete permission checks
and other callers with the old restriction.

The replacement admits a candidate against the current owned address space and
the requested transfer direction. Its extent follows mappings, rather than a
startup heap count, an executable identity, a particular virtual placement or a
fixed number of buffers. Syscall-specific length, record, pathname and staging
limits remain independent contracts. Existing predecessor roots keep their
qualified behavior. A larger constant is not the implementation of this design.

## Portable contract

Candidate start and length are complete target words; their sum must not wrap.
For a nonempty range, every byte must lie in the target's admitted user domain.
The backend must validate the exact current address-space owner and live syscall
entry, and hold a stable observation through the last access. The observation
includes pinned, accessible translation storage and live backing belonging to
the selected mappings. A numeric root address, page-table word or Boolean
supplied by source or serialized metadata cannot construct this authority.

Every translation affecting any covered page must be present and permit user
access. An output transfer additionally needs write permission throughout the
translation path and in the live mapping owner. Readable code pages need not
be writable; execute-disable is not a data-read restriction. A hole, protection
failure, stale mapping or absent backing cannot be treated as a contiguous valid
extent. Table pointers must be checked against the owned accessible translation
storage before dereference. Backing cannot alias translation storage.

An empty extent performs no memory access or page-table read and grants no
usable address. It preserves the existing helper's no-touch behavior, including
for a noncanonical candidate. The syscall still validates its entry, operation
and any independent argument rules. This does not prescribe every Linux
syscall's zero-length result or error order.

Extent admission does not copy data, change mappings or publish partial output.
It is only valid within its original entry/observation lifetime. A caller must
stage and validate input before committing effects, or use its existing
fault/partial-progress contract. Failure before the first copy leaves staging,
user bytes and operation state unchanged. Generated fault recovery remains
required wherever the stable-resident observation does not exclude faults.
Unscoped faults and broken trusted ownership/context invariants remain fatal;
they cannot silently become a successful extent or an ordinary allocation miss.

Stat-family pathname queries admit every required input byte before reading it.
An absent lookup does not use its metadata destination and therefore performs
no output admission or access. Successful lookup admits the complete output
before private staging changes. In the owned selection this replaces only the
preceding stat adapters' fixed pathname placement and unconditional destination
witnesses. Independent selector/flag/lifecycle rules and all predecessor
selections retain their behavior. The [pinned primary-source record](../knowledge/x86_64/newfstatat-research.json)
identifies Linux's lookup-before-copy ordering. This systems-only refinement
introduces no general pathname resolver or core-language change.

Bounded `openat` uses the same complete pathname admission before its first
load. Bounded `getdents64` stages and copies only its selected record extent
after checking descriptor state and count. An EOF result performs no output
admission. The bounded profile-file `read` checks the requested numeric range
before returning EOF. Its check does not consult mappings or grant access.
See the [pinned directory and read source record](../knowledge/x86_64/file-transfer-research.json).

Bounded shell `chdir` checks the complete root pathname before loading it.
The selected `rt_sigaction` paths transfer one complete native action in the
direction of the operation. A terminal request miss performs no transfer.
The file-header read checks its one-byte output before staging or advancing
position. Header poll checks the eight-byte record before reading and the
two-byte `revents` field before writing. The independent shell phase, file
description and signal-selector checks still apply. See the
[pinned shell I/O source record](../knowledge/x86_64/shell-io-research.json).

Owned header polling is a descriptor-readiness operation for either admitted
header form. The clean form may keep its direct reads or poll first. Repeated
polls leave position unchanged; incoming `revents` is ignored, and successful
poll writes `POLLIN` and returns one. A successful header read clears any
pending readiness. Refused admission changes neither position nor readiness.
The external form keeps its existing terminal probes and requires readiness
before reading. The eight-byte input/two-byte output checks remain complete.
Only the current descriptor-three, one-record, `POLLIN`, infinite-timeout shape
is extended. General descriptor/event/timeout polling is still required by the
full kernel goal. Predecessor selections and external digest qualification
remain unchanged. This systems-only refinement uses the standing approval and
changes no core language rule. The [Linux poll documentation](https://man7.org/linux/man-pages/man2/poll.2.html)
defines regular-file readiness and `revents` as output; the existing pinned
source record remains the baseline copy-order evidence.

Bounded shell command reads use the current captured destination and count.
Sealed inputs keep a private cursor until their last byte is delivered. Serial
input may be received before the exact output extent is known. The adapter keeps
that input in a bounded private queue, erases its consumed bytes, and leaves
the shell phase unchanged on a refused copy or short delivery. The queue and
completion facts live in the exec snapshot outside transient argument and
environment staging. Accepted exec and terminal cleanup retire them. Earlier
shell profiles keep their fixed-buffer checks.

The bounded first-image exit validates the completed command and the live
descriptor table after the serial adapter has transferred the old description
into it. A sole-mm exit releases that table and clears its opaque clear-TID
and robust-list registrations while retiring the entire address space.
The first-image route checks that no child exists before relying on that
property. Shared address spaces require their own thread-exit semantics.
Descriptor initialization also transfers the validated signal dispositions into
the process-owned table and clears their prelude slots. Terminal checking
requires those consumed slots to remain empty and retires the current table,
including subsequent signal-action changes. The old slots no longer own signal
state after that transfer.

The first runtime has one serialized process address space, synchronous entry
and no competing mapping editor. Its proof must establish those facts over the
whole check-and-copy interval, including asynchronous entry behavior. Disabled
maskable interrupts alone are not a general concurrency proof. Before adding
preemption, shared address spaces, SMP, demand paging or userfault handling, the
same contract needs actual locks/pins or closed recoverable accesses. Cached
validation across syscalls is forbidden.

## Initial x86-64 realization

The first realization uses ordinary four-level paging, LA48 lower-half user
addresses, 4096-byte leaves and the existing accessible physical window. It
walks PML4, PDPT, directory and leaf entries using each covered virtual page's
indices. It checks present and user permission at every level and write
permission at every level for output. It must validate entry formats and owned
next-table storage before following pointers. Accessed/dirty changes allowed by
the target are not permission changes. The target validates its actual paging
mode and optional permission-control state; it cannot assume that ignored
features are disabled merely because the chosen emulator normally disables them.

The owned-extent profile initializes supervisor write protection before its
first userspace entry and keeps it enabled for the remaining kernel execution.
The transition changes only CR0.WP, preserving every other CR0 bit. Transfers
cannot temporarily disable protection. Earlier profiles retain their qualified
initial mode. Qualification observes the actual before/after registers at the
generated initialization sites; a debugger register edit cannot replace that
guest-side implementation.

Translation storage comes from the actual pinned root/table ownership. The
current bootstrap owns five pages in one extent; that is its present owner
representation, not a portable table-count limit. User backing comes from the
actual image, stack and current frame-backed heap mapping owners. Candidate
bounds do not establish backing ownership. The adapter must bind these owners
to the actual CR3, process context and current entry. A forged present/user PTE
pointing at kernel or table storage cannot manufacture a user mapping.

The target's one-GiB physical-access boundary remains explicit and checked before
table access. It is not a user virtual-address limit. The first component
refuses huge leaves, five-level paging, address masking and additional permission
mechanisms until their own exact mode/ownership/access contracts are implemented.
The full Linux objective still requires their baseline-defined discovery and
behavior; this refusal is not evidence of full UAPI conformance.

The independent model uses owned table pages with level roles and separate
virtual-to-physical backing facts. These stand for already established owners,
not a production API for creating authority. Component tests must bind actual
native observations to this model rather than pass the model's input dictionaries
to the kernel as proof.

## Caller migration

| Area | Existing issue | Required frame-profile migration |
| --- | --- | --- |
| `systems_process_runtime.rs` | Shared readable/writable helpers use `0x619000` and direct leaf selection | Complete owned walk, with no predecessor heap bound |
| `systems_process_entry_start.rs` | Startup paths retain fixed heap comparisons and witnesses | Separate historical startup witnesses from actual subsequent buffer checks |
| `systems_process_entry_read_exec.rs` | Read candidates retain a fixed heap comparison | Validate the complete output extent; preserve descriptor/error order and exec staging |
| `systems_process_entry_files.rs` | Open/read/metadata paths repeat fixed candidate geometry | Validate every input and output before access; preserve private staging and file/inode ownership |
| `systems_process_entry_output.rs` | Write and vector/output paths repeat fixed candidate geometry | Validate every actual input extent, including split pages, before publication |
| `systems_process_entry_completion.rs` | Shell/path/stat adapters and helper emission retain old geometry | Migrate frame-profile candidates and exact write extents; retain predecessor profiles explicitly |
| Shared time, signal, observation, namespace, pipe and child helpers | They call shared extent labels | Preserve caller registers, direction, staged values and failure dispositions when replacing those labels |
| Provider entry helpers | Heap/command validators scan fixed counts or selected leaves | Replace frame-profile range authority with live mapping ownership; fixed witnesses cannot qualify arbitrary buffers |

The implementation review must also inventory direct user loads/stores and
stack-only candidate validators. Searching only for `0x619000` is insufficient.
Fixed ELF-entry and deliberately sealed startup assertions remain legitimate
only for their stated qualification; they cannot constrain general syscall
buffers. Frame-profile selection must reach every relevant emitter, including
files and output, rather than silently using a predecessor helper.

### Shared caller adapter

The x86 shared helper keeps its existing R8 candidate, R9 length and carry-result
convention. In addition to the ordinary six callee registers, its adapter
preserves RDX, RSI, RDI, R8 and R9. A saved RBP supplies a stable debugger CFA
while the actual call stack is aligned, so nested helper calls do not need to
assume one particular entry alignment. Completed saved words, alignment and
return storage are physically cleared before normal helper resumption.

For the initial adapter, the helper return word, saved RBP, five volatile saves,
maximum alignment and bridge words require 88 outer bytes. The gateway keeps
its 48 saved-register bytes and derives the aligned inner retirement window as
`(4096 - 48 - 88) & -16 = 3952`. Its complete physical footprint is therefore
4088 bytes within the original 4096-byte reserve. This is one owning gateway,
with no nested 4096-byte erasure. The native body/call path must separately fit
that derived window. The predecessor keeps its existing 4032-byte inner window.

Admission verifies the generated helper/bridge return sites, actual helper
frame shape and owned continuation-stack extent before lending those outer
bytes. After nonreturning inner cleanup, a generated terminal adapter restores
the original enclosing RBP and clears only its own 80/88-byte outer frame.
An unestablished outer layout cannot authorize that cleanup. These are private
x86 layout choices; the portable operation still requires the same lifetime,
preservation, complete-cost and cleanup evidence on other targets.

Caller migration remains a private component selection until every direct and
shared caller and the complete source/native/load/boot binding are qualified.
Adding the shared adapter alone does not complete this migration or activate
canonical production publication.

## Architecture comparison

The portable operation retains entry, owner, direction, complete extent,
interruption and progress obligations. Table levels, descriptors, privilege
controls and physical-access mechanics remain backend-private. AArch64 must
account for its selected granule, table/block permissions and PAN/UAO policy;
RISC-V must account for the selected translation mode, leaf validity and its
user-access controls. Neither backend should inherit x86 indices, a five-page
root representation, GS, CR3 or a fixed virtual heap address. Neither is granted
an executable qualification by this x86 design.

## Four implementation stages

1. Record the contract, primary-source facts, complete migration boundary and
   independently exercised ownership/translation model. This stage grants no
   new native execution or publication.
2. Implement the actual owned x86 walk and its target binding. Compare native
   component observations with independent range/permission/ownership cases,
   including no speculative dereference of a foreign next table. Account for
   the complete caller/register/stack and retirement costs.
3. Route all actual frame-profile user-buffer callers through that binding.
   Exercise real syscall buffers beyond page 34, split-page and cross-table
   buffers, holes, read-only/supervisor ancestors, overflow, stale ownership and
   check-to-copy stability. Preserve historical profiles and error/side-effect
   order. Qualify the source/native/boot binding before production activation.
4. Qualify the exact committed production sources and rebuilt image, independent
   observations, prior ABI and application regressions and cold reconstruction.
   Update the existing initial-Step-4 PR. General mmap/protect/unmap, hierarchy
   growth, independent process MM and full distribution/container/VM-host
   acceptance remain required after this prerequisite.

Each stage is a commit in the existing PR. Systems-only authority is already
approved; a necessary core-language change would require a separate decision.

Primary facts and their exact source identities are recorded in the
[research record](../knowledge/x86_64/user-memory.md). The formal obligation is
`TOPAL-SYSTEMS-USER-EXTENT-001`.
