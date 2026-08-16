Read a task definition in the same order as its lifecycle. State fields establish
the invariant, `start` establishes the initial state, ordinary handlers describe
the public event and request surface, and `terminate` describes final cleanup.

The compiler does not assign runtime meaning to declaration order, so a
deviation remains valid Topal. The linter reports it as a style warning and
suggests the expected section; it does not reorder code automatically because
comments and declaration-local explanation must move intentionally too.
