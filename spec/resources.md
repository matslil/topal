# Resource semantics

## Formal text

### TOPAL-RESOURCE-OWN-001 — Unique ownership obligations

Every resource identity shall have exactly one live ownership obligation. A
fresh declaration creates it for one binding and one lifetime; redeclaring the
same identity while that obligation exists is invalid.

### TOPAL-RESOURCE-MOVE-001 — Affine movement

Moving a resource shall transfer its existing obligation to the destination
binding and invalidate the source without copying resource identity or state.
Only the current owner may move it, and use after move or destruction is an
error before an external interaction occurs.

### TOPAL-RESOURCE-CLEANUP-001 — Deterministic destruction

Every live obligation shall be destroyed exactly once on every scope exit,
including success, error, termination, and cancellation. Unless dependencies
impose a stronger order, destruction is reverse declaration order. A moved
resource is destroyed through its current owner, not its invalidated source.
