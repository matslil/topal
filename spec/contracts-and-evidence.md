# Contract and evidence semantics

## Formal text

### TOPAL-FUNCTION-CLAUSE-PLACEMENT-001 — Revisioned function clauses

In language revision `v0.2`, a function header SHALL contain, in order, its
parameter list, optional `requires`, optional `effects`, optional `guarantees`,
an arrow, either an unnamed result classifier or `binding : classifier`, and
optional `ensures`. Each clause SHALL occur at most once. A pre-arrow clause
SHALL NOT refer to the result binding. Only `ensures` introduces and may refer
to that binding. The binding SHALL NOT be visible in the body.

Parameter-specific classifiers, including `Exclusive` and `Consumes`, SHALL
remain in their parameter declaration. The rule applies independently to every
nested function classifier. Revision `v0.1` retains
`-> result-classifier : effect-or-resource-classifier`; `v0.2` SHALL reject
that post-result function classifier.

### TOPAL-CONTRACT-REQUIRES-001 — Proven call precondition

`requires P` SHALL be pure and total. A call is valid only if verified evidence
entails `P` after substituting its evaluated arguments and visible static
identities. Missing, refuted, or merely trusted evidence SHALL reject the call
before function entry. Dynamic validation SHALL remain an explicit
`Result`-producing operation and SHALL NOT be inserted as hidden failure.

### TOPAL-CONTRACT-ENSURES-001 — Complete return relation

`ensures Q` SHALL be pure and total and SHALL hold for every implicit,
explicit, successful, and failing return after substituting original inputs,
the named result, and the declared effect trace. A verified relation MAY be
erased. A diagnostic evaluation SHALL NOT replace proof or change a conforming
program's result.

### TOPAL-CONTRACT-INVARIANT-001 — Owned state invariant

A nominal stored type or task definition MAY contain one `invariant binding P`
after state fields and before operations. The binding denotes its complete
value or state only within `P`. Construction SHALL establish `P`; every public
transition SHALL preserve it. An invariant spanning suspension SHALL require
verified version, transaction, or protocol evidence.

### TOPAL-EVIDENCE-KIND-001 — Semantic and implementation separation

Every evidence record SHALL be either semantic or implementation evidence.
Semantic evidence MAY participate in program admission. Implementation evidence
SHALL classify one semantically valid implementation and SHALL NOT change
values, errors, effects, ordering, or protocol traces. Forgetting implementation
evidence MAY make a hard implementation requirement unavailable and SHALL
otherwise preserve typing and meaning.

### TOPAL-EVIDENCE-STATUS-001 — Evidence status

Every retained record SHALL have exactly one status: `verified`,
`trusted-unverified`, `externally-assumed`, or `refuted`. Only `verified`
evidence SHALL discharge memory, bounds, ownership, totality, race, deadlock,
protocol, information-flow, or implementation obligations. Project policy MAY
admit trusted ordinary semantic laws. Externally assumed evidence applies only
when every recorded assumption is explicitly admitted by the consuming
application. Refuted evidence never applies.

### TOPAL-EVIDENCE-NAME-001 — Language-owned property vocabulary

`v0.2` SHALL introduce its standard property constructors unqualified in the
root scope. They are statically resolved objects, not keywords. A conflicting
root declaration SHALL be diagnosed. A library-qualified object with the same
text has no language-defined verification or optimizer meaning.

### TOPAL-EVIDENCE-PRODUCER-001 — Evidence authority

Every record SHALL name its derivation or provider. Source authors, generated
source, and foreign translators MAY request properties and define checked
relations but SHALL NOT assign evidence status or produce implementation,
topology, specialization, or protected-safety evidence. A checker, compiler,
concrete runtime, or typed checked-provider interface MAY produce only evidence
within its declared authority. Analysis tools MAY inspect or validate but SHALL
NOT promote a record by metadata mutation.

### TOPAL-EVIDENCE-BOUNDARY-001 — Complete retained identity

Public and opaque evidence SHALL retain property identity, classified subject,
static parameters, kind, status, producer, assumptions, language revision, and
optional architecture-model identity. Import, substitution, and re-export
SHALL preserve those fields. Architecture evidence and implementation plans
SHALL have no ordinary source constructor or value-level introspection form.

### TOPAL-EVIDENCE-ASSUMPTION-001 — Conditional conclusions

A conclusion derived from external assumptions SHALL retain the transitive
union of their identities and environmental scope. A consumer SHALL either
admit the complete set or treat the conclusion as unavailable. No export,
diagnostic suppression, test success, or foreign annotation SHALL remove an
assumption or convert it to unconditional verification.

### TOPAL-IMPL-EVIDENCE-001 — Opaque architecture seam

Implementation evidence MAY record an optional architecture-model identity and
provider certificate. Ordinary source SHALL request the associated property,
not construct or inspect this record. Until an approved model exists, physical
timing, placement, transfer, scheduler, instruction, compartment, and
fault-domain properties SHALL remain unavailable to hard matching.

### TOPAL-IMPL-SELECTION-001 — Hard and preferred matching

Every hard `guarantees` item SHALL have matching admissible evidence for the
selected complete implementation. `Prefer` SHALL rank semantically valid
candidates lexicographically and MAY fall back when no preferred evidence is
available. Selection SHALL NOT change semantic overload applicability.

### TOPAL-IMPL-UNAVAILABLE-001 — Fail-closed diagnostic

Failure of a hard implementation requirement SHALL reject selection and report
the property, subject, known or unknown derived fact, assumptions, and
considered candidates. A tool SHALL NOT ignore, weaken, or silently reinterpret
the requirement.
