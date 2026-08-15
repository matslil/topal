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
