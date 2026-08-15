# Debugger scripting language variant

The source debugger evaluates commands as Topal code under a domain-specific
language variant. Interactive prompt evaluation selects the variant implicitly.
A command file explicitly selects it and thereby identifies itself as a
debugger script:

```topal
use language (
  version is v0.1,
  features is ( debug )
)

break 42
continue
```

The variant adds debugger functions and capabilities under `lang debug`.
Evaluation begins in that namespace, so complete unqualified command names are
concise. Normal structured Topal qualification remains available and resolves
the same stable function identities. The variant is unavailable to the
debugged application and does not make application introspection equivalent to
debugger control.

Debugger commands are functions. Their arguments and results use ordinary
Topal types, and Topal scripts may compose them with `lang trace` observers.
This permits source locations, function identities, types, values, and derived
semantic events to be constructed and passed without a second debugger
language.

Interactive input adds a user-interface resolution layer. It accepts a unique
prefix or recursively finds one unique command below `lang debug`; ambiguous
input lists candidates. The debugger resolves the input to a canonical Topal
application before evaluation. Command files use strict Topal lookup and do not
accept these prompt shortcuts. History records both entered text and its
canonical form, and exported scripts contain the canonical form.

The same language-variant mechanism may later construct lint-rule and build
script contexts. Each variant adds only its own qualified vocabulary and
capabilities; it does not add another parser or silently change ordinary source
authority.
