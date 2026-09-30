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

Source-level suppression shall name the stable structured identity and remain
effective when project policy changes the finding between warning and error.
One-statement and matched lexical-region suppression shall follow the shared
diagnostic-control semantics.

Every shipped configurable lint diagnostic shall have conformance coverage for
both an explicit command-line disable selector and a source-level stable
identity control. The coverage inventory shall fail when a catalog rule gains a
diagnostic without both cases. Shared syntax and language errors shall remain
visible after an attempted source-level diagnostic control.

## TOPAL-LINT-RULE-001 — Contained Topal rules

Lint rules shall select the `lint` language variant and receive only their
declared, versioned, read-only token, syntax, semantic, dependency, or supplied
trace views. Execution shall be deterministic and resource-bounded, without
ambient filesystem, network, process, debugger, or application authority.

## TOPAL-LINT-DESIGN-PATTERN-001 — Bounded pattern evidence

Design-pattern rules shall receive only their declared revision of the
`design-pattern-syntax` view. Revision one shall derive bounded facts from
parsed declarations, expressions, calls, decisions, lexical scope, and source
spans, then supply an attachment only its declared facts. Its initial AP-03
projection supplies whether an applicable state-named Boolean binding or state
field exists and whether an applicable state-named `Union` has at least eight
alternatives. Its AP-04 projection supplies whether a protocol-named Boolean
binding or state field exists and whether a protocol-named `Union` exists. It
shall provide no application values, ambient authority,
timing, architecture, device, or runtime facts. A structural advisory shall
identify its bounded source evidence and false-positive boundary. A limitation
warning shall identify the catalog condition not observable from the declared
view. No rule shall infer or claim a real-time, safety, security,
distributed-system, or hardware guarantee from source resemblance alone.

## TOPAL-LINT-VARIANT-001 — Rule-module admission

Before execution, a Topal lint-rule module shall explicitly select the `lint`
language feature, use a supported source language version, and expose its named
entry point as a static function. Lint-rule modules shall not combine `lint`
with the privileged `debug` feature. Admission shall parse through the shared
source and syntax layers and fail before loading catalog or application state.

## TOPAL-LINT-FIX-001 — Safe rectification

The linter shall expose a rectification when an entry supplies one. Automatic
mode shall reject overlapping edits, shall not silently apply review-required
changes, and shall reparse and recheck modified source before success.

## TOPAL-LINT-ADAPTER-001 — Consistent interfaces

Terminal, JSON, SARIF, and LSP presentations shall adapt the same findings
without changing identity or severity semantics.
