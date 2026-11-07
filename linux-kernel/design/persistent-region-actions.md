# Ordinary region actions on persistent kernel state

`TK-DEC-272`; systems-only refinement under standing authorization.

Extend [ordinary borrowed-region functions](source-kernel-region-functions.md)
beyond the [qualified root loan](../step-4-root-region-invocation.md). The next
qualification executes an ordinary Topal function on an existing persistent
kernel field during actual file operations. Its source graph performs checked
arithmetic and byte mutation; the owner resumes only after complete temporary
reclamation. This increment includes real syscall integration and physical
evidence in the same qualification.

## Declared callable interface

A separate `borrowed-byte-region-action-v1` source-object context selects one
published ordinary `(KernelByteRegion, Nat, Nat) -> Boolean` export by declared
signature. The two immutable scalar inputs are independently admitted finite
native values in 0..2^64-1, constructed as ordinary Nat values by the private
adapter. Source arithmetic retains arbitrary precision. Source parameter names
carry their ordinary meaning; they do not select a native Linux algorithm.

The source graph uses the existing checked count/load/store/window operations,
ordinary functions, Boolean decisions, numeric algorithms and owner-relative
effects. Declaration-only symbolic selection, module visibility, nonescape,
nonrecursive closed calls and complete bounds retain their current laws. The
new profile admits at most 64 selected source functions, 256 issued views,
4096 complete source/caller stack bytes and a 65536-byte temporary arena, with
the existing 256-bit numeric proof budget. The two transported inputs and all
helper/argument evaluation are charged in the complete certificate.

The old one-argument Unit source object and root declaration retain their exact
contracts. The action context has its own private ABI, exports, runtime/helper
namespace and allocation/fatal hooks. Its linked code and storage remain
unambiguous alongside the root object and existing numeric policy libraries.
Normal application analysis gains no sealed region vocabulary or constructor.

## Owner, domain and field loan

The kernel controller owns the shared descriptor/description store. Descriptor
aliases and parent/child references grant service rights; they do not grant
plain access to its storage. An exclusive controller-domain token permits a
contained loan of the actual non-atomic eight-byte description-position field.
Source receives neither adjacent kind/reference-count/control fields nor a
native pointer or user-memory capability. Effects conservatively identify the
whole controller owner across aliases and contained views.

All controller access is suspended during the source invocation. Every writer,
including native seek/close/exec and inactive-process teardown, participates in
the same domain. The first qualified context serializes those operations on the
single processor which executes the current process services, with interrupts
masked and no callback, suspension or concurrent AP/interrupt access. Masking
local interrupts alone does not establish exclusion against another processor.
A future concurrent service must supply its own complete synchronization-domain
and owner-lifetime evidence before using this loan.

Before entry, validate the actual live controller/description, regular-file
kind, field window, rights, code, stack context, disjoint temporary storage and
fresh nonwrapping generation. Numeric identities and addresses validate an
existing grant; they never create it. No owning resource escapes through the
source view, scalar inputs or Boolean result. Device, DMA, atomic, firmware,
user-transfer and ordinary core Memory/layout protocols retain their domains.
Core resource destruction and AllocationRegion policy retain their meanings.

## Source action in file transfers

The systems root may select `file-position-library`, using the new generic
interface with source/export, stack-limit 4096 and runtime-limit 65536, and no
retained source-state reserve. Selection binds the action into immutable-file
read and volatile regular-file read/write. Other description kinds retain
their existing handlers. With no selection, the existing sealed implementation
remains an explicit compatibility control.

The native service admits the request, rights, complete user prefix, transfer
quantum, page/capacity/clock conditions and effective transfer base through its
existing contract. After successful data transfer, it lends the actual position
field and supplies that effective base and the actual transferred count. An
append write supplies current EOF as its base even when the old position is
elsewhere. Zero/EOF/error paths issue no position action. Explicit-offset
pread/pwrite issue no action and never change the shared description position.

Ordinary Topal source validates the complete eight-byte/value domain, computes
base + count without truncation, verifies the signed file-offset limit and
encodes the resulting little-endian word into that same field. A reusable word
codec uses checked byte operations; it establishes complete extent/value and
access conditions before its first mutation. Its structured ordinary refusals
are tested unchanged. The byte encoding is an explicit internal contract, not
a new core Layout or native word-store operation.

The selected native ordinary-transfer path invokes the source action instead
of performing a second addition/store or a native fallback. After reclamation,
it reads the actual resulting field when subsequent size publication needs the
end position. Existing append placement, clipping, user copying and Linux error
ordering remain in the native service in this first migration. Seeking,
directory cursors and positional-transfer end calculations retain their current
implementations. This increment does not claim a complete source VFS.

The controller keeps operation-private state unavailable to other services and
user execution through the commit. In-place payload/position writes within that
exclusive operation remain private preparation; observable owner publication
follows source completion and reclamation. All recoverable admission failures
precede mutation. A valid admitted transfer requires the selected action to
return true. False or invalid evidence at that point is an invariant failure
which never resumes the controller; it does not invent a new Linux errno or
roll back earlier successful stores. A generic Boolean result alone does not
promise transactional rollback or prove the application's arithmetic contract.

## Invocation storage and completion

The action caller owns distinct arena/control/history storage. It does not
borrow the process numeric arena or the still-live root loan's history. Every
call binds code, actual field owner/window and a fresh generation. The native
hooks meet their inspected leaf/stack contracts without an ambient allocation
service. The caller proves its actual kernel-stack reserve and saves live
caller data outside the area lent to source.

On completion revoke every source view, erase and read back the entire arena,
control and retired helper-stack reserve, clear transient pointer bindings,
restore required registers/RSP and transport the actual Boolean result before
owner access resumes. Successful field bytes survive. Only a separately owned
nonwrapping scalar generation history may persist between calls. Last-owner
termination retires it after active/inactive process references and outstanding
loans are gone. Fatal failure never resumes a suspended owner.

## Models, binding and qualification

The VFS reference model continues to define the independent Linux contract for
positions, append, aliases, errors and teardown. It does not claim to execute a
selected compiled source body. The ownership model records actual controller/
field loans, exclusion, code/generation, reclamation and functional refinement
obligations. A byte-level action model uses the existing owner's actual state;
a detached copied bootstrap buffer cannot qualify persistent field mutation.
Opaque source effects are not silently treated as unchanged or statically
predicted bytes. Source/type/effect checking alone does not prove application-
specific output. Actual source execution and independent byte/position oracles
discharge that separate implementation obligation.

Publication binds frozen sources/modules, declared signature/effects, native
scalar/result transport, complete costs/stack/views, exact hooks, namespaces,
linked code/storage and caller-domain/owner integration. Boot packaging checks
the same binding. Missing/extra selections, changed body/metadata, namespace
collisions and undercharged or incompatible bounds reject before publication.

Qualification observes ordinary helper/entry/native caller frames, both actual
Nat inputs and Boolean result, the real persistent field and adjacent canaries,
shared alias/fork/exec positions, complete caller/arena/control/retired-stack
cleanup before handler resumption, and final active/inactive retirement. Include
append with a different old position, boundary/overflow/read-only refusals,
wrong source and binding controls, source absence, noninvocation on positional/
zero/EOF/error paths and the full current VFS/BusyBox/root-source cohort.

X86-64 implements the first private call/stack/domain/encoding adapters.
AArch64 and RISC-V retain the same source signatures, Nat/Boolean meanings,
plain-memory ownership and lifecycle laws, with separately qualified native
transport, exclusion and storage evidence. No x86 register or GS convention
becomes source meaning.

Risk is high at shared lifetime/exclusion, actual persistent state, commit/error
ordering, scalar ABI, complete bounds and cleanup. Review follows authoritative
resources/effects/layout intent and kernel state architecture, then each owner,
call, source operation, failure and publication boundary. The implementing agent
performs those roles with independent reference/native/physical oracles; human
PR review remains required. New core semantics, if actually required during
formalization, remain a human decision.
