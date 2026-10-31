# Immutable Records in freestanding numeric modules

`TK-DEC-258`; systems-only qualification adopted under standing authorization.
Implemented and physically qualified; see the
[Step 4 report](../step-4-freestanding-numeric-records.md).

The approved architecture places semantic kernel policy in ordinary modules.
`TK-DEC-257` qualifies closed source imports but admits only scalar numeric and
Boolean facts plus the Euclidean pair. This refinement admits ordinary closed
immutable Records composed recursively from those already qualified values.
It preserves the existing Record type, label lookup, construction/evaluation
order, immutable reconstruction and private function-boundary semantics in
`docs/containers.md`, `TOPAL-TYPE-PRODUCT-001` and
`TOPAL-TYPE-RECONSTRUCT-001`. No core-language meaning changes.

The first executable use groups the existing relative-sleep inputs into a
nested request Record, then moves the full deadline policy into its ordinary
imported clock module. The scalar native entry/result interface is unchanged.
The source algorithm uses named fields and immutable reconstruction; it receives
no native layout, context, address, allocator or writable resource authority.
The finite Boolean/Enum root profile and unqualified cross-unit nominal types
retain their previous boundaries.

Admission covers Record construction, statically selected fields, immutable
reconstruction, local bindings, decisions and acyclic ordinary helper calls,
including parameter/result transport. Field values must remain in the qualified
recursive type domain. Containers, callables, effects, nominal payloads,
reflection, dynamic field selection and otherwise unsupported operations remain
profile diagnostics. Records may not escape the one numeric invocation; the
external bridge still extracts and validates its scalar result before complete
arena/stack erasure.

The allocation certificate tracks field bounds by semantic label. Construction
evaluates and charges every field once in source order. Projection charges its
receiver once. Reconstruction charges its base once plus every replacement once,
preserving untouched bounds and leaving the base unchanged. Branch joins retain
conservative bounds for each field of the same checked shape. Cache abstraction
may omit presentation order only because this profile admits no order-observing
operation; ordinary lowering still carries the actual order metadata unchanged.
Record aggregation introduces no new allocation algorithm. The selected shared
SSA/aggregate lowering and actual object closure must confirm that fact.

Complete stack proofs include actual aggregate parameter/result transport,
spills, native alignment, return addresses and all imported/runtime callees.
Missing, dynamic or excessive metadata still rejects publication. Numeric
subvalues keep arbitrary-precision meaning and the existing reviewed runtime
certificate. The same exclusive 65536-byte invocation arena and 4096-byte stack
reserve remain; neither bound is increased to accommodate a new body.

Qualification must execute the structured policy through actual Record helper
frames, verify independent native arithmetic and full state/storage erasure,
retain exact closed source provenance and caller-scope isolation, and exercise
nested fields, reconstruction immutability/evaluation cost, branch bounds,
malformed shapes/types and resource exhaustion. Reordering construction fields
must preserve this named-field algorithm while retaining ordinary Record order
metadata. A changed imported helper must still reach its independent rejection
oracle. The full previous runtime corpus remains required on one primary kernel.

Flattening named facts into more positional native arguments would preserve the
current scalar restriction but prevent ordinary structured subsystem interfaces.
Introducing machine-sized Record fields would alter numeric meaning. Reusing
ordinary immutable Records with complete proofs avoids both choices. AArch64
and RISC-V retain the source shape/evaluation laws while requiring their own
aggregate ABI, frame and runtime-storage qualification.

Risk is high at aggregate lowering, label/type identity, evaluation accounting,
call-boundary stack costs and lifetime of nested numeric values. Review follows
the shared Record model, complete abstract evaluation, actual LLVM/object calls,
isolated module checking and terminal reclamation. Persistent subsystem state,
general allocation and the remaining [Step 4 acceptance](../step-4-status.md)
remain outside this refinement and still required.
