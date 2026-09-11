# Effect semantics

## Formal text

### TOPAL-EFFECT-ROW-001 — Canonical effect rows

An effect row shall retain an unordered duplicate-free set of exact effect
identities and at most one polymorphic tail variable. Display order and source
order do not affect row identity.

### TOPAL-EFFECT-COMPOSE-001 — Conservative composition

Sequential or unordered composition shall infer the union of known effects.
Rows with the same polymorphic tail retain it; rows with distinct unresolved
tails require an explicit instantiation and shall not be silently combined.

### TOPAL-EFFECT-CONTAIN-001 — Effect containment

An implementation effect row satisfies an allowed row exactly when every known
implementation effect is allowed and every unresolved implementation tail is
the same tail admitted by the allowed row. Erasing effects or substituting an
unrelated tail is invalid.

### TOPAL-EFFECT-CLAUSE-V02-001 — Pre-arrow effect bound

In revision `v0.2`, `effects E` SHALL occur before the arrow of the function it
classifies. The inferred implementation row SHALL satisfy `E` under
`TOPAL-EFFECT-CONTAIN-001`. The clause is an upper bound and SHALL NOT require
every listed effect to occur. A nested function's clause belongs only to its
own arrow.
