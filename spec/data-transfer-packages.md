# Data-transfer package boundaries

## Formal text

### TOPAL-TRANSFER-PACKAGE-001 — Extended package ownership

The data-transfer interfaces shall be published as separately versioned
packages named `transfer`, `data`, `store`, `network`, and `device`. They shall
not add members to the fundamental `std` namespace. Each package shall declare
the language revision and its own package revision.

### TOPAL-TRANSFER-PACKAGE-002 — Dependency direction

`data` shall own regions, spans, views, and encoding-independent data shapes.
`transfer` may depend on `data` and shall own endpoint, operation, completion,
sequence, and message-transfer protocols. `store`, `network`, and `device` may
depend on both foundation packages. Foundation packages shall not depend on a
specialized binding.

### TOPAL-TRANSFER-PACKAGE-003 — Host authority boundary

Importing any data-transfer package shall grant no host authority. Host
resources shall enter a program only as explicit capabilities supplied by its
embedding application through the versioned host-operation boundary. A path,
address, integer, or serialized value shall not construct such a capability.

### TOPAL-TRANSFER-PACKAGE-004 — Platform-independent meaning

The public packages shall define one observable meaning independent of the
selected native or virtual backend. Platform-specific facilities may refine a
capability or report an unsupported construction, but shall not conditionally
change a portable operation's protocol, result, or failure meaning.

### TOPAL-TRANSFER-PACKAGE-005 — Tool isolation

Static analysis by the language server or linter shall not open, resolve, or
probe an external resource. Debugger replay shall consume recorded semantic
completion observations and shall not repeat the recorded external effect.

