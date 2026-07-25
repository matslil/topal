# Execution, scopes, and totality

This document defines ordinary block evaluation, immutable bindings, return,
recursion, and termination. These rules complete the execution model used by
function bodies, decision actions, generators, destructors, and task handlers.

## Blocks and sequencing

An indented block creates a lexical scope and evaluates its expressions from top
to bottom. A binding becomes available after its initializer has completed.
The value of the final expression is the value of the block:

```topal
hypotenuse is fn ( width : Float, height : Float ) -> Float
  square is width * width + ( height * height )
  square-root square
```

Every preceding expression must either bind a value or have result `Unit`.
Discarding another result requires an explicit discard operation. This prevents
accidentally ignoring `Result`, `Optional`, completion evidence, a generator
return, or another meaningful value.

An empty block has value `Unit`. Cleanup belonging to the scope runs after its
result has been constructed and before that result is delivered to the
enclosing scope.

## Immutable bindings and shadowing

`is` introduces an immutable binding in an ordinary lexical scope. A name
cannot be bound twice in the same scope. A nested scope may shadow an outer
binding; the formatter and diagnostics should make accidental shadowing
visible, and a project may reject it by lint policy.

Producing a changed immutable value requires a new binding:

```topal
updated-person is person with (
  age is person age + 1
)
```

`with` reconstructs the same product type, replacing the named fields and
retaining the others. Every field invariant is checked again, and dependent
fields whose evidence is invalidated must also be supplied or re-established.
It is not mutation and does not change aliases of the original value.

Task fields are different bindings owned by the task context. A handler
statement whose left side resolves to a task field replaces the task's current
field value after constructing and validating the new immutable value. Local
bindings always take precedence, and task-field replacement can be qualified
when a local name would make the target unclear. No reference to an earlier
mutable view survives the replacement or a handler suspension.

## Return and propagation

`return expression` completes the nearest enclosing ordinary function with the
expression's value. The expression must satisfy the declared output type,
including its `Result` classification. Cleanup for every exited lexical scope
runs in deterministic reverse construction order before the call completes.

`return` is not permitted in a static initializer outside a function, nor
does it return from an enclosing function through an anonymous function.
Each anonymous function, generator, destructor, and task handler establishes
its own return boundary.

Success projection from `Result` performs an early error return from the same
boundary. It follows the identical cleanup rules. Decision-table actions do not
establish separate return boundaries.

## Recursion

A function may refer to itself after its complete input and output type has
been established. A group of declarations may explicitly form a mutually
recursive group; arbitrary forward references do not silently create one.
Every member of the group is checked against the declared types of the others.

Ordinary recursion must be proven terminating. The compiler first recognizes:

- structural recursion on a component of a matched finite recursive value;
- recursion on a constrained integer which moves strictly toward a bound;
- recursion through a finite collection traversal; and
- calls to an already proven terminating function.

When no standard rule succeeds, a declaration may supply a static decreasing
measure into a well-founded order. Every recursive cycle, including a mutual
cycle, must decrease that measure before returning to the same point in the
cycle. The measure is proof information and need not exist at runtime.

Termination checking follows function values. Passing a recursive call through
a higher-order function is accepted only when that function's contract
preserves the required decrease or performs a proven finite number of calls.

There is no unchecked partial ordinary function in safe Topal. A computation
which waits for external interaction may suspend through a declared protocol or
effect. A computation which can produce indefinitely is a generator and must
meet the productivity rule below.

## Generator productivity

Every cycle in a generator must, in finite computation:

- yield a value;
- return or fail;
- suspend on a declared external interaction; or
- observe cancellation from its structured task scope.

A recursive generator call which can occur before any of these boundaries must
decrease a well-founded measure like ordinary recursion. The compiler checks
all decision branches rather than accepting a generator merely because some
path contains `yield`.

Consumers may deliberately traverse an unbounded productive generator. Their
own termination then depends on a statically finite consumer, a stopping
predicate, cancellation, or an external suspension. Materializing an unbounded
generator as a finite container is rejected unless a bound is established.

## Static evaluation

Static functions obey the ordinary termination rules and additionally have no
runtime effects or runtime-only dependencies. Their bound may depend on static
inputs; “static” does not mean constant-time.

The compiler may impose an implementation resource limit on evaluation. Hitting
that limit is a compilation diagnostic, not a possible result of the static
function. It does not weaken a proven termination contract.

## Traversal control

Collection folds, decision tables, and generator consumers provide ordinary
functional control flow without a primitive mutable `while` loop. Early
termination is expressed by the elimination result:

```text
Continue state
Finish result
```

A short-circuiting fold stops on `Finish`; `any`, `all`, `find`, and bounded
generator consumers are standard specializations. Cleanup occurs before the
result leaves the traversal call.

Bidirectional generator drivers explicitly match `Yielded` and `Returned` and
resume the linear continuation with its declared input. Convenience operations
may be added without introducing a second generator state model.

## Decisions still tied to surface syntax

The semantic rules leave open:

- the declaration spelling for a mutually recursive group;
- the spelling for a user-provided decreasing measure;
- the qualified spelling for task-field replacement; and
- whether `with` is the final record-reconstruction keyword.

These are grammar and ergonomics decisions rather than open execution
semantics.
