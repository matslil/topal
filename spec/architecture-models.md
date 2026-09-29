# Architecture-model conformance

## Formal text

### TOPAL-ARCH-SOURCE-001 — Restricted Topal model package

An architecture model SHALL be expressed as a Topal package using the exact
language and architecture-schema revisions declared by its root. Validation
SHALL evaluate only pure, total, deterministic static construction admitted by
that schema. Model evaluation SHALL perform no host discovery, device access,
filesystem search, network access, arbitrary effect, or target-code execution.
Ordinary application source SHALL NOT import the selected model as a semantic
value.

### TOPAL-ARCH-ROOT-001 — Complete model root

The canonical root SHALL contain `schema`, `language`, `name`, `imports`,
`components`, `connections`, `facts`, `costs`, `provenance`, and
`qualifications`. Each collection SHALL have unique stable identities. A
required field with an unknown schema meaning SHALL reject the model; an
explicitly ignorable extension SHALL be retained but excluded from effective
facts and canonical identity only when its defining schema says so.

### TOPAL-ARCH-LAYER-001 — Distinct target layers

Every component SHALL have exactly one kind from `ExecutionArchitecture`,
`AbiPlatform`, `Compute`, `Memory`, `Translation`, `Cache`, `Interconnect`,
`TransferEngine`, `Channel`, `Device`, or `Board`. References MAY connect
layers, but an ISA, vendor, microarchitecture, ABI, operating platform, and
board SHALL retain distinct identities.

### TOPAL-ARCH-FEATURE-001 — Feature compatibility

An execution-architecture feature SHALL name its architecture, revision,
dependencies, exclusions, and semantic facility. A model SHALL reject missing
dependencies, simultaneous exclusions, dependency cycles, or a feature outside
the selected architecture. Optional features SHALL NOT be inferred from vendor
or processor text.

### TOPAL-ARCH-COMPONENT-001 — Component instances

A reusable component definition and each board instance SHALL have different
identities. An instance SHALL name its definition, count, topology parent, and
every overridden parameter. Counts and capacities SHALL be positive. An
override SHALL name an overridable field and SHALL NOT weaken an architectural
legality or safety constraint.

### TOPAL-ARCH-CONNECTION-001 — Typed topology

A connection SHALL name existing endpoints, direction, carried address or
message classes, route resources, and applicable ordering and failure facts.
Every route used by a fact or cost SHALL be a directed connected path. Cycles
MAY describe a physical network but SHALL NOT be interpreted as a zero-cost or
unbounded-capacity route.

### TOPAL-ARCH-MEMORY-001 — Memory and translation facts

A memory component SHALL state capacity, addressability, supported access
sizes, alignment and boundary rules, permissions, visibility and coherence
domains, and accessible initiators. Bank, row, rank, channel, burst,
persistence, ECC, and failure facts SHALL be explicit when relied upon.
Translation SHALL separately identify page sizes, walk paths, translation
caches, address-space identity, and IOMMU or pinning requirements.

### TOPAL-ARCH-CACHE-001 — Cache and false-sharing basis

A cache SHALL name what it caches, attachment and sharing scope, capacity,
line/sector geometry, indexing information known to the provider, associativity,
write/allocation/inclusion behavior, miss and writeback paths, and coherence
participation. Cache utilization and false sharing SHALL be derived using a
program plan; they SHALL NOT appear as unconditional component facts.

### TOPAL-ARCH-ORDER-001 — Ordering and barriers

An ordering operation SHALL name its read/write class, strength, affected
memory or device domains, synchronization scope, preconditions, completion
guarantees, and cost reference. Compiler barriers, memory fences, execution
barriers, cache/translation maintenance, and device/DMA ownership barriers
SHALL remain distinct.

### TOPAL-ARCH-TRANSFER-001 — DMA and channel completeness

A transfer engine SHALL name reachable source/destination spaces, descriptor
and address limits, granularity, alignment, maximum size, scatter/gather and
queue limits, concurrency, completion, failure, coherence, ownership, and
translation requirements. A channel SHALL state payload capacity,
backpressure, ordering, atomicity, visibility, notification, and endpoint
topology. Missing correctness information makes the mechanism unavailable; a
cost estimate SHALL NOT fill it in.

### TOPAL-ARCH-COMPUTE-001 — Compute and instruction facts

A compute component SHALL name its execution architecture and supported feature
set. Instruction facts SHALL distinguish semantic legality, operand/result
constraints, encoding bytes, latency, initiation interval, resource occupancy,
register constraints, and memory effects. SIMD, scalable-vector, SIMT, tensor,
matrix, launch, synchronization, and completion facilities SHALL be represented
only when supplied by that component.

### TOPAL-ARCH-COST-001 — Conditional cost record

A cost record SHALL name its subject, dimensions, conditions, resource and
topology identities, provenance, and applicability. Each dimension SHALL use a
declared exact unit and one of `Exact`, `Interval`, `Estimate`, `Unknown`, or
`Unsupported`. An interval SHALL have ordered bounds. An estimate SHALL retain
its error or confidence description and SHALL NOT satisfy a hard upper bound
unless qualification explicitly establishes a conservative bound.

### TOPAL-ARCH-COST-COMPOSE-001 — Dimension-aware composition

Latency, initiation interval, throughput, capacity, occupancy, code size,
memory, transfer, energy, and predictability SHALL remain distinct dimensions.
Sequential dependent latency MAY add; parallel latency SHALL follow the
dependency graph; traffic sharing a named bottleneck SHALL share its capacity;
and overlapping resource occupancy SHALL be checked against the resource.
Unknown SHALL propagate unless an independent conservative bound applies.

### TOPAL-ARCH-PROVENANCE-001 — Fact provenance

Every legality, topology, cost, calibration, and qualification record SHALL
name one immutable provenance record containing provider, source class,
revision or date, digest, applicability, and assumptions. A measurement SHALL
add hardware, firmware, operating mode, tool, method, workload shape, sample,
and tolerance identity. Measurement SHALL NOT rewrite a reusable base fact.

### TOPAL-ARCH-OVERLAY-001 — Explicit overlays

An overlay SHALL name the exact canonical base identity. Every replacement
SHALL name one existing replaceable fact and preserve its dimension and subject;
every addition SHALL use a new identity. Conflicting replacements, deletion of
required legality, or application to another base SHALL reject the overlay.
The effective model receives a new canonical identity.

### TOPAL-ARCH-CANONICAL-001 — Canonical identity

Canonical model bytes SHALL encode schema and language revisions, normalized
UTF-8 identities, imports by canonical digest, effective components,
connections, facts, costs, provenance, qualifications, and overlay history.
Maps SHALL be arrays sorted by canonical identity; integers and quantities
SHALL use minimal exact encodings; duplicate fields and identities SHALL be
rejected. Model identity SHALL be the lowercase SHA-256 digest of those bytes.

### TOPAL-ARCH-VALID-001 — Ordered validation

Validation SHALL proceed in this order:

1. parse shared Topal source and validate restricted static construction;
2. validate revisions, root fields, canonical ordering, and unique identities;
3. resolve exact imports and overlay bases;
4. validate feature closure and component-local invariants;
5. resolve topology and validate routes, scopes, and addressability;
6. validate legality and ordering facts;
7. validate cost units, bounds, conditions, resources, and provenance;
8. validate qualifications and authority; and
9. derive and verify canonical identity.

Failure SHALL reject the complete model before backend selection. A consumer
SHALL NOT retain facts from a partially valid model.

### TOPAL-ARCH-QUALIFICATION-001 — Separate qualification scopes

Qualification SHALL separately name `Schema`, `Implementation`, `Cost`, or
`Platform`, the model digest, provider authority, compiler/backend version,
covered subjects, assumptions, validation method, and expiration or revision
condition. Schema validity alone SHALL NOT authorize code generation; cost
qualification SHALL NOT establish instruction or platform legality.

### TOPAL-ARCH-TARGET-001 — Target selection

Once a compiler admits multiple qualified profiles, its implicit target SHALL
resolve only the generic baseline for the compilation host architecture family
and platform. A specific target SHALL use the exact selected qualified model.
A cross target SHALL use only explicitly selected foreign model facts and SHALL
NOT inherit compilation-host features or costs. Every artifact and plan SHALL
record the resolved model digest and target layers.

### TOPAL-ARCH-NATIVE-001 — Explicit native detection

Native tuning SHALL require an explicit mode and a qualified detector whose
output names the detected architecture, features, processor, platform,
assumptions, and provider version. Unsupported, contradictory, or heterogeneous
results SHALL fall back to a compatible generic model or reject according to
declared policy; they SHALL NOT silently enable an instruction.

### TOPAL-ARCH-MULTIVERSION-001 — Qualified runtime variants

Runtime multiversioning SHALL retain a qualified generic fallback. Every
specialized version SHALL state its required feature predicate, identical Topal
semantic identity, model identity, and code/resource cost. Dispatch SHALL use a
qualified platform mechanism, select only a satisfied predicate, and introduce
no implicit foreign runtime or loader.

### TOPAL-ARCH-DIAGNOSTIC-001 — Missing-information projection

When an unknown or unqualified fact changes selection or prevents a hard
obligation, the diagnostic SHALL name the source location and transformation,
model and policy identities, missing fact and authority class, legal
alternatives, conservative result, and whether program evidence or provider
evidence can resolve it. The projection SHALL NOT be accepted as model, plan,
or proof input.
