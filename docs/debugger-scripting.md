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

The interactive terminal offers command completion and describes the expected
argument after a complete command. Up and down navigate command history, and
Ctrl-R searches it backward. History contains one copy of a command: entering
an older command again moves it to the newest position. A blank line repeats
the most recent command that advances execution, but does not repeat inspection,
help, breakpoint management, or other non-progressing commands.

The debugger prints the first parsed Topal clause as the initial source
position, excluding launcher metadata such as a hashbang. A built-in clause
such as `use language` has no source implementation to enter, so `step` and
`next` both advance from it to the following clause. `continue` runs until a
breakpoint, watchpoint, diagnostic, manual interruption, or normal application
completion. `run` reconstructs the debuggee and stops at its initial position.
Bare `until` leaves the current source frame; at application scope that means
running to completion. `until LINE`, `until FILE:LINE`, and `until CONDITION`
stop at a location or when a read-only Boolean inspection expression becomes
true.

Interactive semantic history describes rules in ordinary words rather than
showing repository requirement identifiers. Debugger command files retain the
stable identifiers in their deterministic output because that output is an
automation and conformance interface rather than terminal presentation.

The same language-variant mechanism may later construct lint-rule and build
script contexts. Each variant adds only its own qualified vocabulary and
capabilities; it does not add another parser or silently change ordinary source
authority.

## Source frames and help

`step` enters source selected by a `use` clause, while `next` remains in the
current source file. Once inside dependency source, `next`, `reverse-next`, and
`finish` operate on that dependency frame; `finish` returns to the calling
clause. Backtraces and breakpoints identify both file and line so identically
numbered lines in different modules remain distinct.

`help COMMAND` explains debugger commands. `help NAME` searches the debuggee,
the complete configured standard-library source tree, and qualified built-in
declarations. Qualified library names follow their source namespace, for
example `help std web http response`.

A diagnostic raised while advancing the debuggee does not close the command
session. The user may inspect bindings and history or move backward from the
last completed state.
