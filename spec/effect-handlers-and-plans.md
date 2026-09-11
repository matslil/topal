# Effect-handler and implementation-plan semantics

## Formal text

### TOPAL-EFFECT-HANDLE-001 — Closed lexical handler

`EffectProtocol` SHALL declare a closed set of operation inputs, results,
resource identities, and resumption modes. `handle B with H` SHALL select `H`
lexically, require it to implement exactly the handled operations, replace
their abstract effects with handler implementation effects, and propagate all
unmatched effects.

### TOPAL-EFFECT-RESUME-001 — Affine one-shot resumption

`Resumption A R` SHALL be affine and initially live. A handler SHALL either
resume it once with an `A`, abandon it and return an `R`, or return an admitted
explicit error. A second resume, escape beyond the handle scope, or use after
abandonment SHALL be rejected before continuation execution.

### TOPAL-EFFECT-CLEANUP-001 — Abandoned continuation cleanup

Abandoning a resumption SHALL destroy every continuation-owned resource exactly
once in deterministic dependency/reverse-construction order. Handler failure
SHALL retain cleanup failures in the ordinary contextual error chain and SHALL
NOT create an exception channel.

### TOPAL-EFFECT-MULTISHOT-001 — Verified duplication

An operation declared `Multiple` SHALL be handled only when verified
`MultiShot` evidence proves that the captured continuation and retained values
are duplicable, no affine resource is duplicated, and repeated effects are
permitted. Source trust SHALL NOT establish this evidence.

### TOPAL-FUNCTION-SPECIALIZED-001 — Static-input elimination

`Specialized (static-inputs is S)` SHALL be satisfied only when a final
implementation substitutes each named static input and retains no runtime tag,
dictionary, closure, AST node, or dispatch solely attributable to `S`.
`Prefer` MAY select a generic fallback; a hard requirement SHALL fail when
code-shape evidence is unavailable.

### TOPAL-IMPL-PLAN-001 — Compiler-owned typed plan

An implementation plan MAY contain typed fusion, materialization, iteration,
tiling, partition, allocation, reuse, channel, layout, conversion, transfer,
and dependency choices with supporting evidence. Only the compiler/backend
SHALL create or mutate it. Source and tools MAY request a read-only diagnostic
projection which SHALL NOT be accepted as plan or proof input.

### TOPAL-IMPL-REPRODUCIBLE-001 — Reproducible plan identity

A reproducible plan SHALL record language, source-artifact, cost-model, and
optional architecture-model identities plus every decision and verified bound.
Equal inputs SHALL produce a canonically equal projection. Plan choice SHALL
not affect program semantic results.
