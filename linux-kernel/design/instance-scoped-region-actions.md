# Instance-scoped ordinary region actions

`TK-DEC-276`; systems-only refinement under standing authorization. The
[framed transport](framed-region-actions.md) supports different scalar counts,
but its one fixed export, private namespace and hook pair allow only one such
source object in a kernel. Independent owners need independently bound source
instances, including when their ordinary signatures are identical.

## Source and instance identity

Keep the ordinary one-region/zero-to-sixteen-Nat/Boolean selection and the
existing five-argument private frame ABI. Add the separately qualified
`borrowed-byte-region-instance-action-v1` context and a framed source-object
schema with mandatory instance identity. The instance identifies
link ownership and its owning caller; it is neither a core value nor a region,
function-pointer or storage capability. A source path, public export, arity or
string by itself cannot discharge the live caller's ownership proof.

An offline builder may accept a canonical instance label: 1–48 ASCII characters,
starting with a lowercase letter, with lowercase letters, digits and single
internal hyphens only. No final/repeated hyphens, dots, underscores or escaping
forms are admitted. Use a dedicated `topal.region-instance.<label>.` namespace,
outside the Unit, two-scalar action and unscoped framed namespaces. Native
exports/hooks use the label with hyphens replaced by underscores. This mapping
is injective over the admitted labels; labels do not select an instruction set.

The kernel's root and persistent-position callers supply the identities
`root-region` and `file-position` respectively. They are derived from the
qualified caller slots, rather than another source-address constructor or a
user-selected grant. Other standalone instance objects remain object evidence
until a separately qualified caller binds their exact owner/code/lifetime.

Each instance owns its complete source/module snapshot, export, code/read-only
namespace, hooks and call/resource evidence. The native caller additionally
owns its control, temporary arena and generation history. Runtime control code
identity must match the actual selected instance entry. Instances cannot borrow
each other's hooks, arena or owner merely because their layouts/counts match.
Ordinary imported helpers are compiled under their importing instance; a helper
name is not a call to another instance's private native entry.

## Compatibility and representation

Use framed source-object schema 2 with an explicit instance and scalar count.
The private five-argument control/view/u64-frame/Boolean layout remains ABI 1.
Keep the reviewed complete adapter family and exact selected count rendering;
instance naming does not alter ordinary argument order or resource charging.
Bind the naming/instance descriptors together with actual object and linked
instructions/data. Do not infer scope from an untrusted prefix alone.

Unit source schema 3, two-scalar action schema 1 and unscoped framed schema 1
remain independently usable with their exact old names, entry layouts and
omitted instance fields. New instance-only fields must reject under those
contexts, including explicit nulls. New root and field invocation bindings use
distinct schema revisions with exact instance/count facts. Missing, mismatched,
duplicate, extra and foreign-instance namespaces/hooks/entries reject before
publication or boot; selected root and field instances may coexist.

Keep each existing arena/control/history allocation separate. Retired stack
storage may be reused only after the preceding invocation's complete erasure,
under the existing synchronous, nonnested and controller-exclusion laws. This
does not introduce concurrent invocation or claim SMP exclusion from IF=0.

## First kernel integration

Allow `region-library` to select the instance-action context as an alternative
to its unchanged Unit context. That caller expects exactly zero
Nat facts and a Boolean result. Supply a canonical null/zero input frame and
bind the `root-region` instance. The first source wrapper invokes the existing
ordinary Unit byte-copy implementation and returns true; the original Unit
source and its legacy execution remain available.

The root still lends its actual 64-byte bootstrap owner before deriving atomic
access. Source writes remain actual owner writes. On return, retain the real
source Boolean as a native scalar, revoke views and erase/read back complete
arena/control/retired-stack state before any owner resumption. An admitted false
result then takes the invariant-fatal path, preserving successful owner stores
without promising rollback or a fabricated successful result. Both the Boolean
acceptance check and the existing application-specific byte guard are runtime
obligations; the opaque model must not predict either source result or bytes.

The file-position declaration may select the new instance-action context; its
existing unscoped framed and two-scalar action contexts remain selectable.
The `file-position` instance retains exactly its effective-base/count facts,
actual shared field, immutable 32-byte staging, complete cleanup and true-only
admitted-transfer contract. Its ordinary source and Linux behavior do not
change. The primary kernel must execute both framed instances with separate
code/hook identities and disjoint arenas/controls/histories. Earlier Unit,
two-scalar action and unscoped framed controls remain explicit.

## Bounds and target obligations

Retain the 65536-byte temporary arena, 4096-byte retired-source-stack reserve,
256-view, 64-function and 256-bit numeric proof limits. Every instance charges
64 root-view bytes, at most 24 bytes per fact, the 32-byte header and complete
source evaluation. Bind actual adapter/helper/caller frames, return addresses,
alignment and staging. Root result preservation cannot hide an uncharged live
temporary or erase a still-live root frame. Measurements remain separate from
the complete static bounds.
The new x86-64 root binding charges its 56 live caller bytes in addition to
source depth and the existing 32-byte alignment allowance. Legacy Unit bindings
retain their existing representation and qualification.

Instance identity, source selection, value semantics and owner-relative effects
are architecture independent. AArch64 and RISC-V require their own native name,
layout, calling, code/stack and physical qualification. Only x86-64 is
implemented here; neither a register assignment nor a Linux operation becomes
source language meaning.

## Qualification and alternatives

Link and execute at least two independent framed source objects with the same
arity and public export, plus the existing profiles, without symbol or data
collisions. Observe instance-local ordinary parameters, allocations, writes,
generations and cleanup. Cross-instance code/control calls must reject before
source entry/allocation/owner mutation. Reject malformed/colliding labels,
wrong instance/count, altered actual code/data/hooks and incompatible/extra
metadata. Preserve valid empty-data zero-input graphs and canonical legacy
serialization.

Physical kernel evidence must cover both real owner domains, null/zero root
transport, source/root/helper frames, actual Boolean results, live field words,
caller preservation, full cleanup and terminal history retirement. Include a
root false-after-store control, existing wrong-byte/pre-entry/absence controls,
separate legacy Unit/action/unscoped-frame runs and the complete current
application/policy cohort. Controlled observations wait for GDB completion and
prove the guest remains stopped before teardown.

Names based only on arity collide for distinct owners with identical signatures.
A shared global dispatcher/arena obscures the existing independent ownership
domains. Adding an ABI for each kernel operation exposes policy rather than the
common callable mechanism. Instance-owned linking and callers preserve one
frame ABI while making those ownership boundaries explicit.

Risk is high at instance/caller identity, namespace closure, cross-owner aliasing,
legacy formats, Boolean failure sequencing and complete stack/cleanup bounds.
Review source/value/effect/resource intent, exact object/link/caller identities,
all transition/failure paths and independent native/physical oracles. Human PR
review remains required; no independent reviewer result is claimed. General
persistent allocation and the remaining [initial acceptance work](../step-4-status.md)
remain separate obligations.
