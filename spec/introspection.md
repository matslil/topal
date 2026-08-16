# Static-introspection semantics

This specification formalizes the source-visible static introspection described
by `docs/introspection.md`. Introspection values exist in the static phase and
do not imply retention of runtime reflection metadata.

## TOPAL-INTRO-QUALIFIED-001 — Qualified operations

The identifiers `view`, `identity`, `declaration`, `context`, `version`, and
`public-members` SHALL be introspection operations only when selected through
the compiler-provided `lang` scope. In infix position, `same-object`,
`equivalent-type`, `compatible-with`, and `same-layout` SHALL likewise require
the explicit `lang` qualifier. A tool SHALL NOT introduce an unqualified
binding for any of these operations.

## TOPAL-INTRO-STATIC-001 — Static-only evaluation

Every introspection operand SHALL be a statically known language object of the
kind required by the selected overload. Every introspection result SHALL be a
typed static value. A conforming implementation SHALL reject an attempt to
inspect an arbitrary runtime value and SHALL NOT retain descriptors solely to
support runtime reflection.

`lang trace` is not an exception granting runtime reflection to ordinary code.
It SHALL statically construct an observational task whose declared typed inputs
are supplied by the trace system. The observer SHALL NOT inspect runtime values
or events outside those inputs.

## TOPAL-INTRO-TRACE-001 — Typed trace observers

`lang trace` SHALL construct a deterministic observational task over declared
fundamental or derived event inputs and static configuration. Returning a
typed event value SHALL emit that value as one derived event; returning `None`
SHALL emit no event. Observer inspection SHALL NOT itself emit value or
function events. Observer state and output SHALL depend only on its initial
configuration and ordered inputs, and observer execution SHALL NOT alter the
observed application's values, dependencies, errors, authority, or scheduling.

An implementation MAY realize an observer as a task, filter, state machine,
debugger evaluator, or replay transform when the resulting typed stream is
equivalent. Dependencies between derived observers SHALL be acyclic.

## TOPAL-INTRO-VIEW-001 — Kind-preserving views

`lang view` SHALL select its result type from the semantic kind of its operand.
Type, Function, Scope, Constraint, Effect, and Protocol operands SHALL produce
their corresponding typed view described in `docs/introspection.md`. Views
SHALL preserve recursive identity, dependent structure, visibility, and
opacity; they SHALL NOT expose representation layout or compiler-generated
instructions.

## TOPAL-INTRO-DECLARATION-001 — Declaration metadata

`lang declaration object` SHALL produce a `lang DeclarationView` for the
visible declaration by which `object` is known at the inspection site. Missing
metadata SHALL remain absent. The view SHALL NOT expose private source text,
private declarations, or an unrestricted syntax tree.

## TOPAL-INTRO-CONTEXT-001 — Language context

`lang context` SHALL produce the language name, the source location's active
numeric `Version`, and its selected feature set as a `lang LanguageContext`.
`lang version` SHALL produce that same `Version` value, not a runtime String.

## TOPAL-INTRO-RELATION-001 — Explicit semantic relations

For statically known operands, `lang same-object` SHALL compare stable
language-object identity, `lang equivalent-type` SHALL compare semantic type
inhabitants and constraints, `lang compatible-with` SHALL apply the applicable
boundary compatibility relation, and `lang same-layout` SHALL compare explicit
external Layout values. A tool SHALL reject operands of an inapplicable kind;
in particular, two runtime values without language-object identities SHALL not
compare as the same object merely because both identities are unavailable.
