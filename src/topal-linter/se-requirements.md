# Topal linter requirements

## TOPAL-LINT-CATALOG-001 — Best-practice selection

The linter shall select version-compatible best-practices by stable identity,
owned namespace, and tag. Exact-identity settings shall override broader
settings. Installation of an external database shall neither execute nor
enable it; project selection shall be explicit.

## TOPAL-LINT-DIAGNOSTIC-001 — Shared findings

Findings shall use the shared compiler/interpreter diagnostic model and add the
best-practice identity and version. Template and recommended entries normally
warn; best-practice entries normally produce errors. Projects may override the
severity or disable any selected entry.

## TOPAL-LINT-RULE-001 — Contained Topal rules

Lint rules shall select the `lint` language variant and receive only their
declared, versioned, read-only token, syntax, semantic, dependency, or supplied
trace views. Execution shall be deterministic and resource-bounded, without
ambient filesystem, network, process, debugger, or application authority.

## TOPAL-LINT-FIX-001 — Safe rectification

The linter shall expose a rectification when an entry supplies one. Automatic
mode shall reject overlapping edits, shall not silently apply review-required
changes, and shall reparse and recheck modified source before success.

## TOPAL-LINT-ADAPTER-001 — Consistent interfaces

Terminal, JSON, SARIF, and LSP presentations shall adapt the same findings
without changing identity or severity semantics.
