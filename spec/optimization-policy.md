# Optimization-policy conformance

## Formal text

Let `C` be the finite candidate-plan set for one decision, `F(c)` its semantic
and target feasibility predicates, and `K(c, d)` a qualified cost fact for plan
`c` in dimension `d`. `unknown` is not a numeric value.

### TOPAL-OPT-FEASIBLE-001 — Feasibility precedes ranking

The selectable set SHALL be `S = { c in C | all F(c) }`. `F` SHALL include
semantic preconditions, target legality, language guarantees, applicable
safety, capacity, timing and security constraints, and explicit hard limits.
An optimization profile, goal, cost estimate, or pass override SHALL NOT add a
candidate to `S`.

### TOPAL-OPT-COST-001 — Qualified cost vectors

Every `K(c, d)` SHALL retain its dimension, unit, conditions, provenance,
applicability, uncertainty, and qualification identity. Missing information
SHALL be `unknown`, distinct from zero, infinity, unsupported, and an
unmeasured but bounded value. Costs in unlike dimensions SHALL NOT be added or
converted without an explicit qualified conversion rule.

### TOPAL-OPT-DOMINANCE-001 — Conservative Pareto comparison

For the relevant comparable dimensions, `a` SHALL dominate `b` only when
qualified facts prove `a` no worse than `b` in every dimension and strictly
better in at least one. An unknown or incomparable fact SHALL NOT prove either
relation. A dominated plan SHALL NOT be selected while its dominator remains
in `S`.

### TOPAL-OPT-ORDER-001 — Ordered goals and stable tie-break

Nondominated plans SHALL be compared lexicographically by the effective
ordered goal list. A goal SHALL decide only when qualified evidence proves its
ordering. Applicable source `Prefer` goals SHALL precede compiler-profile soft
goals for the implementation choice they govern. If no goal decides, the
lexicographically least stable canonical plan identity SHALL win. Selection
SHALL NOT depend on candidate enumeration order.

### TOPAL-OPT-PROFILE-001 — Standard profiles

The compiler SHALL recognize `-O0`, `-O1`, `-O2`, `-O3`, `-Os`, and `-Oz` as
the versioned profiles defined by `docs/optimization-policy.md`. `-O0` SHALL
satisfy `TOPAL-COMPILER-O0-001`. Target selection SHALL be independent of the
profile. The artifact SHALL record the effective profile and optimization-plan
revision.

### TOPAL-OPT-TARGET-001 — Target-selection contract

An omitted target SHALL resolve to the qualified generic baseline for the
compilation host's architecture family and platform when that baseline is
implemented. It SHALL NOT implicitly select optional host CPU features. An
explicit target, CPU, board, model, or native selection SHALL validate through
`TOPAL-ARCH-TARGET-001` and SHALL be rejected when incompatible, foreign native
detection is requested, or qualification is insufficient. An incremental
compiler MAY implement only the target admitted by `TOPAL-COMPILER-TARGET-001`.

### TOPAL-OPT-CONTROL-001 — Goal, limit, and pass controls

Repeated explicit goals SHALL replace profile goals in occurrence order.
Repeated limits SHALL add hard predicates to `F`. Stable pass enable and
disable options SHALL update the profile's optional-pass set in occurrence
order. An isolated-pass option SHALL start from the `O0` optional-pass set and
SHALL be incompatible with another isolated selection or explicit standard
profile. Mandatory semantic lowering, validation, verification, and backend
correctness work SHALL NOT be exposed as disableable optional passes.

### TOPAL-OPT-LIST-001 — Optimization discovery

The optimization-list query SHALL require no source input and SHALL list every
known optional pass with stable ID, short description, implementation status,
required evidence, and normally enabling profiles. The query SHALL NOT produce
a native artifact.

### TOPAL-OPT-EXPLAIN-001 — Decision explanation

An optimization explanation SHALL identify the effective target and model,
profile and plan revision, ordered goals, hard limits, enabled passes,
candidates considered, relevant preconditions and cost evidence, rejections,
missing facts, and selection. Its canonical decision content SHALL be
deterministic and SHALL exclude incidental timestamps and output paths.

### TOPAL-OPT-DIAGNOSTIC-001 — Missing-information diagnostics

When a missing fact materially changes or prevents a decision, the compiler
SHALL be able to identify the affected optimization, alternatives,
conservative outcome, missing fact, policy/model identities, and a legitimate
source or target-model provider. Missing cost evidence SHALL NOT establish a
preference. An optimization remark SHALL NOT become a language error unless a
hard requirement is unsatisfied.

### TOPAL-OPT-REPRODUCIBLE-001 — Recorded decision inputs

For identical source and dependency identities, tool identities,
architecture-package identities, explicit options, and admitted calibration
inputs, optimization-plan selection and canonical explanation SHALL be
identical. Host discovery, environment values, profile data, filesystem order,
and runtime measurements SHALL affect selection only when explicitly admitted
and recorded.
