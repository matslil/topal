# Module and namespace semantics

## Formal text

### TOPAL-NAMESPACE-ROOT-001 — Executable root namespace

The identifier `root` in scope-value position shall resolve the current source
session's root namespace. A qualified path beginning `root member` shall resolve
`member` only among declarations in that namespace. Remaining application
operands shall apply the resolved terminal value using its ordinary function,
generator, or value semantics. Binding or displaying the namespace shall not
copy, flatten, or execute its declarations.
