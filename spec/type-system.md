# Type system

## Formal text

### TOPAL-TYPE-KIND-001 — Object kinds

Every static object has exactly one kind in:

`{Value, Type, Function, Predicate, Constraint, Capability, Interface, Pattern,
Effect, Protocol, Scope, Module}`.

`Predicate <: Function`; no other distinct kinds are subkinds. A kind mismatch
is a static error. Types classify values; functions transform objects;
constraints refine values of one base type; capabilities attach verified or
explicitly trusted semantic evidence to existing objects. Types and functions
are never interchangeable. This realizes `TOPAL-REQ-MODEL-001`.

### TOPAL-TYPE-JUDGE-001 — Typing judgment

The primary judgment is `Γ; Κ ⊢ e : T ! ε ▷ φ`, where `Γ` maps names to exact
objects, `Κ` contains canonical capability evidence, `T` is the result type,
`ε` is the conservative effect set, and `φ` is retained relationship evidence.
An implementation shall accept `e` only if one unique derivation exists after
ordered overload selection. Accepted derivations preserve `T`, `ε`, and `φ`.

### TOPAL-TYPE-ID-001 — Type identity

Two structural types are identical iff their constructors, ordered positional
components, labeled components, classifiers, constraints, and retained static
parameters are recursively identical. Two nominal types are identical iff they
originate from the same published declaration and have identical retained
parameters. Aliases preserve identity; new nominal constructions do not.
Visibility may hide structure but never changes identity.

### TOPAL-TYPE-PRODUCT-001 — Products, records, sums, and enums

`Tuple(T₁…Tₙ)` classifies ordered values `(v₁…vₙ)` exactly when each `vᵢ:Tᵢ`.
The zero-field product `Tuple()` is `Unit` and classifies exactly the single
value `()`.
`Record{lᵢ:Tᵢ}` classifies a value containing each label exactly once. Label
order is declaration order and does not affect label lookup. A closed record
admits no additional field; an open record pattern may observe additional
visible fields but neither captures nor reconstructs them.

`Union(T₁…Tₙ)` is untagged and valid only when static matching identifies at
most one member for every admitted value. `Variant{Cᵢ:Tᵢ}` carries exactly one
nominal tag and payload. An enum is a variant whose payloads are `Unit`.

### TOPAL-TYPE-BOOLEAN-001 — Boolean values

`Boolean` classifies exactly the two distinct values `true` and `false`. Neither
value implicitly converts to or from a numeric value. The reserved literal
`true` evaluates to the first value and `false` to the second without name
resolution.

### TOPAL-TYPE-EQUALITY-001 — Equality application

If `T` provides canonical `Equality` evidence and `a:T`, `b:T`, then `a = b`
evaluates to `true` exactly when the two values are equal under that evidence,
and otherwise to `false`. `Unit`, `Boolean`, `Int`, `Rational`, and `String`
provide canonical equality; string equality compares the preserved Unicode
sequence. A tuple provides equality exactly when corresponding fields do, and
compares equal exactly when every corresponding field compares equal.
A record provides equality exactly when both operands have the same labeled
field sequence and corresponding field values provide equality; it compares
equal exactly when every corresponding field compares equal. Records with
different labeled shapes have no shared equality overload.

Canonical conversion may make one equality overload applicable. In particular,
mixed `Int` and `Rational` equality applies `TOPAL-NUM-INT-RATIONAL-CONVERT-001`
once and then uses rational equality. Different types without such a conversion,
and tuples of different arity, have no applicable equality overload rather than
evaluating to `false`. Equality returns `Boolean` and performs no numeric
coercion of that result.

`a != b` has exactly the same applicability, conversions, and observations as
`a = b`, and returns the Boolean negation of that equality result. It is a
derived operation and does not introduce distinct inequality evidence.

### TOPAL-TYPE-ORDERING-001 — Total-order application

If `T` provides canonical `TotalOrder` evidence and `a:T`, `b:T`, their
three-way comparison produces exactly `Less`, `Equal`, or `Greater`, with
`Equal` agreeing with `TOPAL-TYPE-EQUALITY-001`. The predicates `<`, `>`, `<=`,
and `>=` select the corresponding result or result set and return `Boolean`.

`Tuple(T₁…Tₙ)` provides `TotalOrder` exactly when every `Tᵢ` provides it.
Tuple comparison is lexicographic from the first field and stops at the first
non-`Equal` result. Tuples of different arity have different types and no shared
tuple-ordering overload. Canonical field conversions may make the corresponding
field comparison applicable before the tuple result is selected.

### TOPAL-TYPE-CONSTRAINT-001 — Constraints

For base type `T` and total pure predicate `p:T→Boolean`, constraint `C=(T,p)`
classifies exactly `{v∈T | p(v)=true}`. Static construction with a statically
known `v` succeeds only when the compiler proves `p(v)`; otherwise it is a
static error. Dynamic validation returns `Result(C, Codes)` and yields reusable
evidence on success. Constraint evidence may be forgotten losslessly to `T`;
the reverse is validation, not conversion.

### TOPAL-TYPE-CAP-001 — Capability evidence

`Κ ⊢ o : C[e]` means object `o` satisfies capability `C` using evidence `e`.
For each canonical pair `(o,C)`, at most one evidence interpretation exists.
Evidence is admitted only when claimed by the definition context of `o` or `C`,
or derived by exactly one visible derivation. An explicit owner claim suppresses
derivation; competing derivations are an error. Import order cannot select
evidence.

Capability laws have trust state `verified`, `trusted-unverified`, or `refuted`.
Refuted evidence is rejected. Evidence required for memory safety, totality,
race freedom, or deadlock freedom shall be `verified`. Suppression cannot alter
trust state.

### TOPAL-TYPE-MATCH-001 — Header and pattern matching

A function header is a static pattern over its input object. Matching proceeds
left-to-right. A name first binds the complete matched object; recurrence of the
same name requires exact identity. `_` checks its position without binding.
Classifier chains refine left-to-right. A successful match makes all retained
relationships and visible evidence available to the body.

An open structural record pattern matches an anonymous record containing at
least the named visible fields. A nominal record matches only through an
explicitly published structural view. Hidden declarations never participate.

### TOPAL-TYPE-CALL-001 — Function application

If `Γ;Κ ⊢ f : Function(A→B, εf)`, `Γ;Κ ⊢ a:A ! εa`, and the first applicable
overload header matches `a`, then:

`Γ;Κ ⊢ f a : B[a] ! (εa ∪ εf[a]) ▷ φf[a]`.

Substitution retains dependent identities, constraints, sizes, versions,
states, and existential witnesses. An expected result may check the selected
result but never changes overload order.

### TOPAL-TYPE-CONVERT-001 — Conversions

Implicit conversion is permitted only for the single canonical conversion
declared lossless for an exact source and destination pair. At most one such
path may exist and implicit conversion never chains. Lossy, rounding,
saturating, validation, representation, and effectful transformations require
explicit functions. Conversion does not unify repeated pattern bindings.

### TOPAL-TYPE-EXIST-001 — Existential results

`exists X:K. T[X]` packages a witness `x:K` and value `v:T[x]`. Elimination
introduces a fresh rigid identity for `x`; it may be used through revealed
constraints but cannot escape a scope unless the result type repackages it.
Finite dynamic alternatives preserve a sum of exact implementation evidence;
erasure retains only common guarantees and the union of possible effects.

### TOPAL-TYPE-TOTAL-001 — Totality and failure

Ordinary functions shall prove that every input reaches a value in finite
computation. Structural recursion must decrease verified well-founded evidence.
Potentially infinite production is admitted only as a productive generator in
which every request yields, ends, fails, closes, terminates, or suspends on a
declared external event in finite computation. Runtime failure is an explicit
`Error` in `Result(Value,Codes)`; exceptions are absent. This realizes
`TOPAL-REQ-TOTAL-001`.

### TOPAL-TYPE-SOUND-001 — Preservation and progress

For safe closed `e`, if `∅;Κ ⊢ e:T ! ε` and `e → e'`, then `∅;Κ ⊢ e':T ! ε'`
with `ε' ⊆ ε`. Either `e` is a value, is a declared external suspension, or a
unique permitted step exists. No permitted step produces an unclassified
value, undeclared effect, or undefined operation. Implementations may reject a
program when required proof is unavailable; they may not accept it by assuming
unverified safety evidence. This realizes `TOPAL-REQ-SAFE-001`.

## Graphical presentation

```mermaid
flowchart TD
    E[Expression] --> N[Resolve visible names]
    N --> M[Match headers in source order]
    M -->|no unique derivation| R[Reject]
    M --> C[Collect canonical capability evidence]
    C -->|missing, competing, or unsafe trust| R
    C --> T[Derive exact type and relationships]
    T --> F[Infer conservative effects]
    F --> P[Check totality and safety obligations]
    P -->|unproved required obligation| R
    P --> A[Accepted typed term]
```

## Explanatory notes

The type system deliberately distinguishes value restrictions from semantic
promises: constraints select values, while capabilities provide evidence about
objects and operations. “First applicable overload” is deterministic even when
later declarations would be more specific.

This specification permits conservative rejection. That freedom does not allow
tools to disagree about the meaning of a program they accept: exact types,
retained relationships, effects, and selected declarations remain fixed.
Representation layout, serialized encoding, memory access, and task scheduling
are governed by their respective specifications and cannot be inferred from
type identity alone.
