# Topal formal specification

This directory defines the normative semantics of the current Topal design
revision. Human-readable intent comes from [`docs/`](../docs/), while core
system constraints come from [`se/`](../se/). Where the design deliberately
defers a feature, that feature is outside the accepted language until a later
revision specifies it.

The specification is divided into:

- [language syntax](syntax.md);
- [type system](type-system.md);
- [numeric semantics](numbers.md);
- [generic export intermediate language](generic-ir.md);
- [native serialization protocol](serialization.md);
- [memory model](memory-model.md); and
- [concurrency model](concurrency-model.md).

Each normative rule has a stable ID. **Shall**, **must**, and **is** are
normative; **should** is a recommendation; **may** grants permission. Mermaid
diagrams are informative views of the preceding formal text. Explanatory final
sections are informative unless they cite a normative rule.

## Revision and conformance

These documents describe revision `design-0`. A source, artifact, or protocol
participant shall declare the revision it implements when it crosses a tool or
storage boundary. A conforming implementation shall either implement every
applicable rule or reject the revision or feature before processing it. Silent
fallback is forbidden.

## Common notation

- `x ∈ S` means that `x` is a member of set `S`.
- `A ::= ...` defines grammar production `A`.
- `Γ ⊢ e : T ! ε` means environment `Γ` assigns expression `e` type `T` and
  effect set `ε`.
- `Σ; Γ ⊢ e ⇓ v; τ` means store model `Σ` and environment `Γ` evaluate `e` to
  value `v` with observable trace `τ`.
- `R⁺` is transitive closure; `R*` is reflexive transitive closure.
- `⊥` denotes rejection, not a runtime value.

Rule IDs are never reused. Replaced rules remain recorded as retired and point
to their replacements.
