# Resource, exclusivity, and region evidence

## Formal text

### TOPAL-PERF-DIMENSION-001 — Portable resource dimensions

`ResourceBound` SHALL support `Work`, `Span`, `AllocationTotal`, `PeakLive`,
`Retained`, `Stack`, `QueueEntries`, `TransferCount`, and `CodeSize` with an
exact subject scope and static pure total bound expression. `OExec` and
`OAlloc` SHALL remain asymptotic `Work` and `AllocationTotal` shorthands;
`NoAlloc` SHALL remain exact. Unknown SHALL be distinct from infinity and SHALL
NOT satisfy a hard bound.

### TOPAL-PERF-COMPOSE-001 — Dimension-specific composition

Sequential work, allocation total, and dependent span SHALL add; alternatives
SHALL take a conservative maximum; independent span SHALL take a maximum.
Peak-live and retained storage SHALL be derived from lifetime overlap. Queue
entries, transfers, stack, and code size SHALL compose only within their exact
named scope. Overflow or an unavailable layout SHALL produce unknown rather
than a smaller bound.

### TOPAL-PERF-PREFER-001 — Soft resource selection

An unwrapped resource requirement is hard. `Prefer (G1, ..., Gn)` SHALL compare
applicable implementations lexicographically by provable satisfaction, using
source order as the final tie breaker. It SHALL NOT manufacture evidence,
admit an unsafe implementation, or change the program's semantic result.

### TOPAL-PERF-PROGRESS-001 — Closed progress classes

The progress vocabulary SHALL be `MayBlock`, `ObstructionFree`, `LockFree`, and
`WaitFree (maximum-own-steps is B)`. It classifies a complete interaction
implementation and lists scheduling and completion assumptions. Only a
compiler, concrete runtime, or later approved provider MAY establish
nonblocking progress. Progress SHALL NOT imply elapsed-time response.

### TOPAL-VALUE-EXCLUSIVE-001 — Inferred invocation exclusivity

`input : T : Exclusive` SHALL require verified last-use, nonescape, alias,
lifetime, and span-disjointness evidence for that invocation. It SHALL permit
representation reuse without exposing mutation and SHALL NOT become a
permanent property of `T`. Programmer trust and foreign annotations SHALL NOT
establish it.

### TOPAL-VALUE-CONSUMES-001 — Explicit consumption

`input : T : Consumes` SHALL transfer the complete semantic version on
successful entry and make every later caller use of that version invalid. The
operation MAY return a new value sharing reused representation, but the source
contract SHALL expose neither reference mutation nor allocator identity.

### TOPAL-RESOURCE-REGION-001 — Scoped allocation region

Every allocation through `Allocate R` SHALL record a storage dependency on the
live `AllocationRegion R`. Region cleanup SHALL occur once and deterministically
after all dependent values have ended. An implementation MAY select stack,
arena, scratch, or individual storage only when observable results and declared
bounds are preserved.

### TOPAL-RESOURCE-ESCAPE-001 — Region escape prevention

A region-dependent value SHALL leave its scope only with the complete moved
region, with verified promotion or copy into an enclosing lifetime, or with
proof that it has no runtime storage dependency. All other returns, captures,
messages, or stores SHALL be rejected before cleanup.
