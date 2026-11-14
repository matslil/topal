# Kernel binding for the ordinary frame policy

This continues TK-DEC-282 Stage 3 under the approved systems-only direction.
The [ordinary policy](frame-metadata-policy.md) supplies selection and metadata
effects. Connecting it to a kernel requires the actual allocator owner, rather
than a larger bootstrap byte probe or a numeric result treated as authority.

An optional `frame-allocator-library` root declaration selects the exact
`physical-frame-policy-tail-v1` interface. Its ordinary entry has
`(KernelByteRegion, Nat, Nat, Nat) -> Nat`, with the first Nat as countdown.
Runtime/stack reserves remain 65536/4096; the declaration does not turn either
temporary reserve into a persistent metadata capacity. Persistent metadata and
live-extent capacity derive from the owned normalized memory description.
The slot is unique and belongs to the root's consumed memory description and
its single `FrameAllocatorContext` creation. It is separate from the bootstrap
region, namespace, page-pool and other source-library slots.

The source checker retains the obligation and its context/lifecycle. A compiler
binding must retain the complete root and contained primary/module snapshots,
check the exact ordinary interface/termination/effects and rebuild its complete
source/native closure. Neither the declaration, a program-only API, serialized
metadata nor the standalone native component grants kernel publication.
Missing, extra, wrong-slot and mismatched inputs must be refused. Existing roots
without this declaration keep their existing qualification.

Allocator creation must validate the complete normalized allocatable topology
and every reservation, including the actual linked image, metadata/handle
backing, handoff/resource bytes, translation state and invocation domains. An
old numeric floor is not a reservation proof. Sparse topology and provenance
barriers must be retained. Construction must reject an unsupported topology or
physical-access geometry rather than silently drop owned allocatable ranges.

Each allocation receives the actual runtime count/alignment, admits identity
capacity before mutation, lends the actual table exclusively, and runs ordinary
source with a checked work budget. The native owner checks the complete result,
the whole permitted metadata delta, reservations, provenance and disjointness
from all live extents. Zeroing the complete physical extent precedes grant
publication. Only success publishes a new nonwrapping extent identity; ordinary
exhaustion preserves allocation cells and the issuance counter.

Release first verifies the actual affine handle's allocator instance, identity,
whole count/alignment/provenance and lack of live mappings or other loans. It
runs source validation/release and consumes the whole handle once. Foreign,
stale, duplicate, partial or still-mapped handles cannot authorize source
mutation. Numeric physical bases used inside the target provider remain private
implementation facts, not source handle constructors.

The owning call binds the actual privileged context, critical/preemption lease,
stack domain and disjoint mapped metadata/history/private/output/recovery
storage. Bootstrap and syscall stacks require their own actual context proof.
The existing 64-byte root adapter's fixed stack check cannot establish it.
Normal return requires complete inner and owning-caller physical retirement;
failed source stores/protocol checks preserve performed writes and issuance,
poison the allocator and enter nonreturning trusted fatal handling. An all-ones
source result after writes is not ordinary exhaustion and cannot cause rollback.

Allocation and release must replace the actual provider/root lifetime paths,
including mapping ownership transfers and construction/destruction. Workload
memory requests must reach this allocator; substituting constant boot probes
does not qualify general allocation. Publication/boot packaging must reconstruct
and compare all allocated source/provider/caller/data contributions and load
permissions against fresh inputs. Privileged VM observations must exercise the
real owner, physical zeroing, overlapping/reserved refusal, simultaneous extents,
exhaustion/reuse and whole normal/fatal retirement before canonical publication.

The shared contract remains opaque frame ownership. AArch64 and RISC-V require
their own memory inputs, granules, physical-access, ordering and stack/context
providers. Implementation and execution qualification remain x86-64.

## Fresh source and native reconstruction

Canonical construction must start from the immutable root and its actual frame
library/module snapshots. The source check produces a private checked input for
the frame Root and memory provider; it does not remove the frame declaration
to select a predecessor provider. Program-only APIs continue to refuse that
selection. Other selected libraries retain their own source/native checks.

Reconstruction rebuilds the source, runtime, owner, private boundary, frame
controller, physical/context target, memory dispatch, owning gateway, Root and
provider. It derives the combined source/controller/syscall stack bound from
these actual objects, including the caller's alignment and return storage.
Every allocated contribution receives exact ownership endpoints. Binding must
compare all relocated bytes, symbols, zero storage, permissions, alignments and
actual load owners against fresh objects, and verify that the allocator's image
reservation equals the complete allocated image end. Candidate metadata cannot
replace any of these checks.

A frame reconstruction record remains insufficient on its own: publication and
boot must also reconstruct and bind every other selected source library, the
actual entry/target, source and artifact identities, and the boot image/resource
relationship. Unsupported combinations of independent tail-owner namespaces
must be refused until they have disjoint qualified realizations; duplicate
native names cannot silently select another component's owner.

The source-bearing artifact application route takes the frame snapshot separately
from the other library slots and retains the declaration in its checked program.
It rebuilds all native inputs, compares the metadata producers' original library
objects, and links the reconstructed contributions. Linking measures the complete
allocated image, relinks with that reservation and rejects changing geometry.
Only a successful complete source/native/load binding can select this larger
reservation; a symbol or supplied manifest cannot relax a predecessor profile.

Published provenance retains the complete root/frame/other-library/module
snapshots, generated frame objects and LLVM inputs, original producer identities
and observed load/caller binding. Boot packaging reconstructs all those sources
with the exact qualified toolchain and compares the entire fresh binding and
artifact associations before accepting the image. It also retains the other
library/owner validators. Mixed, missing, extra, duplicate, null-erased, stale or
orphan source/native data must be refused. Program-only entry points remain
insufficient; the source-bearing application route supplies the full inputs.

## Native bootstrap component under construction

The native controller uses one private 128-byte allocator context, a comparison
snapshot and a live registry. Each registry record retains issuer, extent ID,
start index, whole count, alignment, provenance, mapping-pin count and live
state. The private 24-byte token contains issuer, extent ID and registry index.
These representations are target implementation details, not Topal address or
handle constructors. Resolution writes physical facts only into a separately
admitted provider-owned result; it does not mint source ownership or byte access.

The x86 bootstrap target constructs the actual table from the complete E820
boundary set and explicit kernel/bootstrap and boot-resource reservations.
It classifies each interval before aligning inward, including overlapping RAM
claims: prematurely merging identical RAM claims could erase an unaligned
boundary and disagree with the reference normalization rule. All E820 RAM
claims share the same type-1 provider provenance. Unknown and non-RAM claims
exclude overlapping intervals. Empty or overflowing claims and extended
handoffs outside this implementation are refused.

The component's bootstrap physical-access geometry is the existing 1 GiB
identity map. Any RAM claim extending outside it is an unsupported handoff,
even when a later normalized overlap might hide that claim. Construction
refuses that geometry rather than truncating the memory description. Backing
arrays cover this architectural window; the actual table, token and registry
capacities use the derived allocatable frame count. No application-slot budget
limits those capacities. Expanding physical access remains required for the
general kernel.

The linker must bind the complete allocated image end, including metadata,
registry, token and temporary backing. In this bootstrap layout the initial
0–16 MiB reservation and contiguous image backing above it form the complete
reserved prefix; the boot resource is separately reserved. A new layout cannot
reuse this prefix argument without proving its actual reservations. The
experimental linker relinks with the measured allocated end and checks that
the resulting load geometry did not change. This is component evidence, not
the canonical source/native/load reconstruction required for publication.

Admission checks the actual bootstrap CPL, interrupt flag and stack window,
the unique context and exact owned-domain addresses, lifecycle, clean source
temporaries and identity budgets. This does not admit syscall stacks. The
zeroing callback checks pending issuance and all selected cells before writing
the actual identity-mapped physical bytes. It cannot publish the token; the
controller publishes only after complete zeroing returns successfully.

The guarded process controller tests and the privileged bootstrap component
have distinct scopes. The latter must compare its real E820-derived table
against the reference normalization model, separately exercise sparse and
unaligned controlled handoffs, and observe physical zeroing before publication
and on reuse. Neither fixture executes userspace. Their success does not
replace production root/workload integration.

The bootstrap target's terminal handler retires source domains and halts with
the allocator poisoned. Complete owning-caller stack/recovery retirement and
its reconstructed native bound are still required. This component alone cannot open the public compiler or artifact gates: the
gateway, real root lifetime paths and complete publication reconstruction must
be qualified together by the source-bearing route described above.

## Owning bootstrap gateway

The subsequent unpublished gateway owns one invocation lease and its complete
call storage. Its private operations are creation, allocation, whole release,
mapping-pin change and provider-fact resolution. It captures the actual boot
stack and admits CPL0, IF0 and DF0 before lending an invocation. The target
variant requires the live gateway lease and verifies its current stack lies in
that actual invocation domain. A busy or invalid entry cannot overwrite an
existing lease or claim cleanup authority for it. This is a bootstrap binding;
syscall contexts need their own typed stack/critical-lease binding.

The machine gateway preserves all six callee-saved registers and clears the
other scalar argument registers before resumption. Native call-graph evidence
includes the dispatch and a conservative source-owner terminal-callback
contribution. The body, call/alignment bytes, saved registers and return word
must fit the unchanged 4096-byte bound. Physical retirement covers 4032 scratch
bytes and 48 saved-register bytes; the 48-byte recovery record is separately
retired. Late failures use the restored stack/register state and never reload
erased saved-register copies.

Source abort, controller protocol failure and post-controller cleanup failure
all preserve the metadata/registry/token/issuance effects which actually
occurred, poison the allocator, reclaim source and caller temporaries and halt.
An identity already issued before a later failure must not be rolled back.
The private 32-byte provider-fact result belongs to the caller on normal
resolution; it is distinct from the retired source facts/output frames. A
terminal path retires that result too, since its caller cannot resume.

Component observations must inspect normal and fatal physical retirement,
including a late failure after saved-register retirement, and unwind through
the gateway while source/checkpoint/completion bodies use the separate private
stack. These are prerequisites to binding actual production root allocation,
release and mapping lifetimes. This gateway alone does not grant canonical
publication or general workload acceptance.

## Unpublished full boot Root integration

The next experimental binding lowers the complete existing x86 boot IR through
the owning gateway. It retains one private 24-byte value slot per affine IR
allocation definition. These slots store handles; they do not limit the native
registry or normalized allocatable topology. The linked slot domain is reserved
with the complete image and checked separately from the native token pool.

Allocation resolves private physical facts and immediately erases the 32-byte
result after consuming them. Release calls the real allocator before clearing
retained registers. The compiler tracks ownership transfers for construction
payload/metadata/hierarchy, published stack loans, retained prepared-process
stack, user mappings and anonymous backing. Each actual loan pins its native
extent; returning the matching owner unpins it before whole release. Translation
setup uses an allocated five-page hierarchy. Active map edits use an allocated
two-page hierarchy and release it only after unmap commit and TLB invalidation.
The active bootstrap CR3 hierarchy remains live and pinned at boot completion.

An unpublished full-root VM test uses the complete firmware map and a clearly
identified static-ELF ABI fixture. Its independent observations check topology
completeness, issued handles, disjoint physical zeroing, real mapping pins,
release, caller retirement and the final active hierarchy. This still does not
qualify dynamic workload allocation, syscall-stack leases, canonical artifact
reconstruction, real distribution applications, containers or VM hosting. The source-bearing route must meet the complete publication binding
obligations independently of this fixture.

The [runtime-memory binding](dynamic-frame-memory.md) continues through actual
syscall request counts and exec/exit retirement, with a separately checked
syscall context inside the owning gateway.
