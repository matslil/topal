# Source transition of retained process-identity state

`TK-DEC-262`; systems-only refinement adopted under standing authorization.
Design adopted; implementation and physical qualification remain required.

Step 2 puts kernel state policy in ordinary Topal modules over owned resources.
Syscall selection and header/time policy now execute in source, but process-ID
selection and its retained next-ID state still change in native code. Move that
existing bounded transition into source, and qualify extraction of a flat numeric
Record result. This establishes a value-level state transition boundary without
claiming general persistent source allocation or effectful subsystems.

## Ordinary state and result

Add optional `process-identity-library` with versioned interface
`process-identity-transition-v1`, the unchanged 4096-byte stack limit and
65536-byte temporary-runtime limit. Its ordinary declared export has type:

```text
Record (next : Nat) -> Record (allocated : Nat, next : Nat)
```

The input is a copied semantic snapshot of the existing process resource's
next-ID cursor. Legal cursors are zero or 2..2147483647; zero denotes the initial
state. No address, process handle, resource identity, owner or capability enters
the source function. Its exact transition preserves the existing bounded profile:

```text
allocated = 2 if input.next = 0, otherwise input.next
next      = 2 if allocated = 2147483647, otherwise allocated + 1
```

Both outputs lie in 2..2147483647. They are numeric facts, not process authority.
The initial parent retains PID one. Creation still requires the existing empty
child slot and lifetime/argument/resource checks; reuse follows reaping and that
slot's ownership, including wrap to two. This does not establish a general PID
namespace, tunable PID limit or arbitrary process creation.

The common policy belongs in `kernel/policy/` with ordinary imported numeric
helpers. Unlike syscall numbers, this bounded state law has no x86 register or
number-table dependency. Another architecture must qualify its native adapter and
resource representation while preserving the value transition.

## Record extraction and private adaptation

The new result qualification admits exactly the selected interface's flat Record
of primitive numeric fields. Ordinary semantic field identity and declared types
remain authoritative. The compiler shall retain actual construction order in the ordinary per-value
order suffix and record the canonical field slots and suffix layout in artifact
metadata. Extraction shall adapt by logical name, not construction order.
Reversing declared/constructed output order must preserve the same logical result.
This does not admit general aggregate escape, persistent source values, new core
Record semantics, nominal payload interchange, callbacks or effects.

The private native bridge accepts the bounded cursor, validates its shape before
allocation, boxes it through the shared numeric runtime and constructs the
ordinary single-field input Record. It calls source and extracts both actual
result fields. Extraction validates the ordinary numeric representation and exact
protocol range without truncation. Only two native scalar words leave the bridge;
no source pointer, Record storage or allocated numeric object escapes the exclusive
invocation. The first x86-64 realization uses a qualified two-integer private C
return convention, with independent C/native and actual frame evidence.

Source/module snapshots, canonical result slots/order-suffix layout, generated bridge, retained
runtime, native object and resource proofs enter exact artifact provenance.
Unknown, missing, mismatched or unqualified declared source must fail publication
or use; it must not silently select native state arithmetic. Existing roots
without this declaration retain their bounded behavior.

## Owning resource and commit boundary

Invoke source at the current child-identity selection point, after existing
creation/lifetime/argument checks and temporary request staging, before publishing
the child identity or changing the retained next-ID cursor. The process resource
continues to own its cursor, child slot, generation, private memory and native
continuations. Source proposes an immutable value transition; native validation
and the owner's existing construction protocol apply it. Numeric equality of an
ID does not create or replace an owner.

The bridge and caller validate the result protocol, preserve live native facts
and callee/SIMD state, erase/read back the whole arena and exactly retired stack,
clear temporary pointers and expose only the two validated words. Only then may
the existing owner publish the allocated ID and next cursor. Existing argument,
capacity, registration, snapshot, inheritance, scheduling and error/commit order
remain unchanged. A malformed private fact, result, binding, ownership, exhausted
reserve or incomplete erasure fails closed; it is not a new user errno path.

The new policy must operate alone or alongside dispatch/header/deadline policies.
Shared storage permits only sequential exclusive invocations with distinct active
identities and code bindings. Dispatch finishes before child creation invokes
identity policy; no source state crosses invocations through temporary storage.
Exec retains selected bindings and the owned resource's state. Terminal teardown
erases/read backs every binding, temporary allocation and existing backing.

Complete allocation bounds include copied facts and all evaluated source/helper/
bridge work. Complete actual stack bounds include input/output Record transport,
shared runtime callees, native saves, returns, field extraction and alignment.
Measured high-water marks alone do not establish either bound. The current
reserves must remain unchanged.

## Qualification and review

Risk is high at aggregate return adaptation, retained state publication, numeric
identity versus authority, shared-arena lifetime and native continuation safety.
Review follows authoritative Record/resource semantics, the bounds evaluator,
ordinary lowering, actual object/ABI/frame data, independent transition arithmetic
and full source/native storage reclamation.

Required evidence covers initial/ordinary/wrap transitions, malformed private
facts/results, exact structural signatures and field identity/order, real source
input and imported frames, both decoded outputs, preserved owner state before
commit, actual later cursor/child identity publication and reuse, independent
policy selection, all selected bindings and complete regression. A changed helper
must be rejected by an independent transition oracle after cleanup and before
state publication, with native root/provider objects unchanged. Controlled copied
facts may exercise the maximum/wrap boundary while leaving the persistent cursor
unchanged; report that as controlled input, not billions of executed lifetimes.

Implementation and tests are independently authored under the existing licensing
policy and add no third-party implementation material. General persistent
allocation, mutable subsystem interfaces, scalable processes, dynamic loading,
writable storage, networking, isolation, distribution/container/VM-host acceptance
remain required by the [initial plan](../step-4-status.md).
