# Tool diagnostics

### TOPAL-DIAG-MODEL-001 — Shared diagnostic record

Every source-facing Topal tool SHALL represent a diagnostic with one severity,
stable code, message, one-based source line and column, and optional source
excerpt and help. A linter finding SHALL extend that same record with the
stable best-practice identity, entry version, and executable rule version; it
SHALL NOT define a competing finding model.

### TOPAL-DIAG-ADAPTER-001 — Presentation independence

Terminal, JSON, SARIF, and LSP adapters SHALL preserve the shared diagnostic's
severity, code, message, location, help, and best-practice provenance whenever
the target protocol can represent those fields. Structured rectification SHALL
remain distinguishable from explanatory help. Changing an adapter SHALL NOT
change diagnostic selection or program evaluation.
