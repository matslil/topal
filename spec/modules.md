# Module and namespace semantics

## Formal text

### TOPAL-NAMESPACE-ROOT-001 — Executable root namespace

The identifier `root` in scope-value position shall resolve the current source
session's root namespace. A qualified path beginning `root member` shall resolve
`member` only among declarations in that namespace. Remaining application
operands shall apply the resolved terminal value using its ordinary function,
generator, or value semantics. Binding or displaying the namespace shall not
copy, flatten, or execute its declarations.

### TOPAL-NAMESPACE-ALIAS-001 — Immutable namespace aliases

Binding a namespace value with `is` shall create an immutable alias retaining
the original namespace identity, members, visibility, and overload ordering.
`alias member operands` shall resolve `member` within that retained namespace
before applying remaining operands. It shall not copy members into local scope
or combine them with declarations bearing the same unqualified names.

### TOPAL-NAMESPACE-USE-001 — Making a namespace available

`use path` shall resolve `path` to a published namespace and produce that same
namespace value for optional binding. It shall make the qualified path
available without flattening members into the current lexical scope. Applying
`use` to a terminal non-namespace value shall be rejected.

### TOPAL-NAMESPACE-SNAPSHOT-001 — Alias declaration visibility

A namespace value captured by a binding shall contain declarations visible at
that binding statement. A declaration introduced later in the same source
session shall not retroactively enter the earlier namespace value. Resolving
the live `root` namespace later shall observe the later declaration.

### TOPAL-NAMESPACE-OVERLOAD-001 — Qualified overload preservation

Qualified lookup through a namespace or alias shall retain the selected
namespace's complete source-ordered overload set. Application shall choose the
first applicable retained declaration without combining overloads from the
caller's lexical scope.
