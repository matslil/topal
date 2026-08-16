# Best-practice database

### TOPAL-BEST-PRACTICE-IDENTITY-001 — Stable owned identity

Every best-practice entry SHALL have one structured Topal identity owned by its
publishing namespace. Presentation names and storage paths SHALL NOT determine
identity. Deprecated, obsolete, renamed, and replaced entries SHALL retain
resolvable historical identity.

### TOPAL-BEST-PRACTICE-STATUS-001 — Lifecycle status

An entry status SHALL be exactly `proposed`, `active`, `obsolete`, or
`deprecated`. A proposed entry SHALL NOT be enabled by default. Obsolete SHALL
record the first language version for which it no longer applies and SHALL
remain applicable to matching earlier contexts. Deprecated SHALL record an
explanation. Obsolete and deprecated entries MAY identify a replacement and
SHALL NOT be deleted merely to remove findings.

### TOPAL-BEST-PRACTICE-CLASS-001 — Guidance strength

An entry class SHALL be `template`, `recommended`, or `best-practice`.
Template and recommended entries default to warning; best-practice entries
default to error. Every entry SHALL explicitly record default enablement and
severity. Recommended entries SHALL record known exceptions. Promotion to
best-practice SHALL assert that no unavoidable exception is known within the
declared applicability.

### TOPAL-BEST-PRACTICE-APPLICABILITY-001 — Context selection

Applicability SHALL identify language and version range and MAY constrain
required, alternative, or excluded language features, capabilities, source
kinds, and platforms. The catalog SHALL preserve those constraints for
evaluation against an inspected declaration's exact constructed language
context. An obsolete entry SHALL be disabled at and after its obsoleting
version while retaining earlier coverage.

### TOPAL-BEST-PRACTICE-GENERATED-001 — Synchronized projections

Generated human, agent, and lint-catalog projections SHALL remain version
controlled, identify generator/schema/source versions and authoritative input
digest, and be regenerated with every authoritative change. Conformance
validation SHALL fail when regeneration differs from committed output.
An executable lint attachment in the catalog SHALL identify its contained
engine, entry point, independent rule version, required analysis stage, and
stable versioned read-only view, and stable `L-...` diagnostic code. A Topal attachment SHALL embed its source and
SHA-256 digest in the generated catalog. A consumer SHALL verify that digest
and reject an unknown engine or stage before executing the attachment; rule
execution SHALL NOT require ambient access to the catalog producer's files.
Consumers SHALL reject a view name or revision they do not implement before
executing the rule.

### TOPAL-BEST-PRACTICE-RULE-CONTAINMENT-001 — Bounded rule admission

A lint-rule consumer SHALL validate deterministic source-size and syntax-tree
budgets before executing Topal rule code. It SHALL reject declarations or
expressions outside the pure subset implemented for the selected read-only
view. Rejection SHALL precede application-state access and SHALL use a stable
diagnostic code which distinguishes resource exhaustion from unsupported
authority or syntax.

### TOPAL-BEST-PRACTICE-RECTIFICATION-001 — Distinct rectification advice

When an entry declares suggestion rectification, each finding SHALL carry the
suggestion as structured rectification data distinct from general explanatory
help. Presentation adapters SHALL preserve that distinction. A suggestion
SHALL NOT be represented or applied as an automatic source edit.

An automatic rectification SHALL consist of half-open normalized-source spans,
replacement text, and one of `presentation-only`, `syntax-preserving`,
`semantics-proven`, or `review-required`. Default fixing SHALL apply only the
first three classes. Before changing a source file, the tool SHALL reject
invalid or overlapping eligible spans, apply the complete edit set in memory,
reparse and recheck the result, reject a non-convergent edit set, and replace
the file as one transaction.
`review-required` edits SHALL remain unapplied.
