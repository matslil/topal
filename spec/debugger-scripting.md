# Debugger scripting

### TOPAL-DEBUG-LANGUAGE-001 — Debug language variant

A debugger command file SHALL explicitly construct a language context whose
features include `debug`. Interactive debugger prompt evaluation SHALL select
that variant implicitly. The variant SHALL provide debugger operations under
the structured `lang debug` namespace and SHALL NOT grant those capabilities
to the debugged application.

### TOPAL-DEBUG-COMMAND-001 — Function commands

Debugger commands SHALL be typed Topal functions. A debugger script SHALL use
ordinary strict Topal name resolution from the `lang debug` context. An
interactive prompt MAY accept a unique prefix or recursively resolve one unique
name beneath `lang debug`, but SHALL diagnose ambiguity and SHALL resolve input
to a canonical Topal application before evaluation. Exported or replayed
scripts SHALL use the canonical form rather than prompt shortcuts.

### TOPAL-DEBUG-INTERACTIVE-001 — Interactive control interface

An interactive debugger SHALL display the first parsed Topal clause as the
initial source position and SHALL NOT identify launcher metadata as executable
source. `step` and `next` SHALL both advance across a built-in clause that has
no source implementation to enter. The debugger SHALL provide command-name and
argument completion, up/down history navigation, and backward history search.
Command history SHALL contain no duplicate entries; re-entering a command SHALL
move that command to the newest position. Empty input SHALL repeat the latest
execution-progressing command, if any, and SHALL NOT repeat a non-progressing
command.

Forward `continue` SHALL extend live execution until a configured stop,
diagnostic, manual interruption, or application completion. `run` SHALL restart
the debuggee from its initial state. Bare `until` SHALL leave the current source
frame, while an argument SHALL identify a source line, source-qualified line,
or read-only Boolean condition. Interactive semantic output SHALL translate
stable rule identifiers into human-readable descriptions; deterministic script
output MAY retain stable identifiers for conformance automation.
