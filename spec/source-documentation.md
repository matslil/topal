# Source documentation specification

### TOPAL-DOC-LEX-001 — Documentation comment

A source line whose first non-indentation characters are `###` SHALL be tokenized
as documentation trivia through, but not including, its terminating newline.
The marker and at most one following space SHALL not be documentation content.
Ordinary `#` comments SHALL remain ordinary comments.

### TOPAL-DOC-ATTACH-001 — Structural attachment

One or more consecutive documentation-comment lines SHALL form a documentation
block. A block SHALL attach to the next documentable declaration or parameter in
the same enclosing declaration list. Blank lines and ordinary comments MAY
intervene. A block SHALL NOT cross a scope boundary, cross another documentable
declaration, or attach to an identifier occurrence in an expression.

### TOPAL-DOC-TARGET-001 — Documentable declarations

Named functions, generators, types, aliases, constructors, fields, parameters,
capabilities, effects, and other named declarations SHALL be documentable. Each
overload SHALL retain independent documentation. An operator's documentation
SHALL be the documentation of the declaration having that operator identity.

### TOPAL-DOC-VIEW-001 — Declaration metadata

Attached text SHALL be normalized by removing each marker and at most one space,
then joining documentation lines with newline characters. `lang declaration`
SHALL expose that text through `DeclarationView.documentation`. Built-in
declarations SHALL support equivalent metadata.

### TOPAL-DOC-GENERATE-001 — Explicit inputs

The reference-documentation tool SHALL generate reStructuredText for every
explicitly supplied Topal file. For a supplied directory it SHALL include Topal
files directly in that directory. It SHALL visit descendants only when
`--recurse` is supplied. It SHALL NOT add an implicit standard-library path.

### TOPAL-DOC-BUILTIN-001 — Built-in inclusion

The reference-documentation tool SHALL omit built-in declarations by default
and SHALL include the built-in `lang` namespace when its built-in inclusion
argument is supplied. Generated entries SHALL include declaration syntax or
function signatures and all available declaration and parameter documentation.

### TOPAL-DEBUG-HELP-001 — Declaration help

Debugger `help` without an argument SHALL list debugger commands. `help` with an
identifier or qualified path SHALL resolve visible declarations and print their
syntax and documentation. Ambiguous unqualified input SHALL list candidates.
Qualified `lang` built-ins SHALL remain available without changing the
debuggee's imports.
