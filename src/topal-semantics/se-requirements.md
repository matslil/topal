# Shared semantic-model requirements

## TOPAL-SEM-ASSURANCE-001 — Fail-closed evidence and resource models

The shared semantic library shall model evidence kind, status, producer,
subject, assumptions, language revision, and optional architecture identity.
It shall reject unauthorized producers and shall permit externally assumed
facts to discharge only ordinary semantic laws whose complete assumptions the
application admits. It shall provide conservative resource-bound composition,
closed progress ordering, invocation-exclusivity checking, consumption, region
escape checking, and compiler-only implementation-plan authority.

## TOPAL-SEM-PORTABLE-RUNTIME-001 — Deterministic portable reference behavior

The shared semantic library shall provide deterministic, architecture-neutral
reference behavior for synthesized channel selection, retained snapshots,
transaction commit and conflict, clock identity, monotonic observation,
periodic late-release policy, static-rate balance and scheduling, closed effect
protocols, and affine resumptions. Reference behavior shall not claim a
physical schedule, timing bound, nonblocking implementation, or compiler
lowering.

## TOPAL-SEM-LAYOUT-INFOFLOW-001 — Checked representation and policy models

The shared semantic library shall validate shape identities, bounded address
arithmetic, nonoverlapping and aligned strides, blocked storage size, canonical
sparse coordinates, zero-copy compatibility, finite information lattices,
implicit program-counter propagation, and scoped declassification or
endorsement authority. Validation shall fail closed on unknown identities,
overflow, overlap, or authority mismatch.

## TOPAL-SEM-ARCH-001 — Deterministic architecture reference model

The shared semantic library shall validate unique architecture component,
connection, cost, and provenance identities; reject dangling topology,
resources, provenance, feature dependencies, and reversed cost intervals; and
derive an order-independent SHA-256 identity from canonical effective facts.
This reference model shall perform no host detection and shall not itself
qualify a backend, platform, or empirical cost claim.

## TOPAL-SEM-INTEGRATION-001 — Explicit completion boundary

Reference-model unit tests establish the behavior of the shared algorithms but
shall not be reported as complete source-language, interpreter, debugger, or
compiler conformance. Each specification domain remains `planned` in
`se/core-language-coverage.md` until its source forms, tool integrations, and
applicable compiled evidence satisfy the terminal disposition recorded there.
