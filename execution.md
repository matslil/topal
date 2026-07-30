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

## Completion dependencies

`Unit` means that a call produces no result and establishes no dependency on
when its execution finishes. After initiating a function whose declared result
is `Unit`, evaluation may continue when value ownership, effects, and the
structured scope permit it. The compiler may execute the call inline, defer it,
or run it concurrently when those choices are observably equivalent.

`Completed` is the distinct zero-data evidence that execution finished.
Applying a function which returns `Completed` orders a dependent continuation
after that completion without requiring an operating-system thread to block.
A fallible completion returns `Result ( Completed, Errors )`.
`Result ( Unit, Errors )` is invalid because an interaction with no completion
dependency has no result channel on which to report failure.

Unobserved completion does not erase effects or detach computation. Effects of
a `Unit` call remain outstanding and constrain later conflicting interactions
until the work finishes. Its structured scope retains the work through cleanup,
termination, and failure containment. A function which cannot report failure
must handle it internally or route it through a separately declared effect.

These rules apply equally when an [interface](interfaces.md) implementation is
an ordinary function or a task event. The return type supplies the completion
contract; implementation evidence supplies the calling convention and the
compiler's scheduling information.

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

Declaration scopes behave as if they are processed in two stages. The compiler
first collects every declaration whose complete classification is explicit,
then checks its definition with all such declarations in the scope visible.
An implementation may optimize or combine these stages provided observable
name resolution is identical.

Function input and output types are explicit, so later function declarations
may be referenced from earlier definitions. The compiler constructs the call
graph and treats each strongly connected component as one mutually recursive
group. No source-level recursive-group declaration is required. Overload
priority remains source order even though every complete header is visible,
except when an explicit call-site resource `Prefer` construction ranks
applicable implementations before that final tie-breaker.

```topal
even is fn ( value : Nat ) -> Boolean
  value = 0 then true
  otherwise odd ( value - 1 )

odd is fn ( value : Nat ) -> Boolean
  value = 0 then false
  otherwise even ( value - 1 )
```

This visibility does not make initializer values available before construction.
Ordinary evaluated bindings retain their dependency and sequencing rules, and
a cycle which requires an initializer's value before that initializer completes
is invalid. Declaration visibility also does not cross lexical or namespace
boundaries.

Ordinary recursion must be proven terminating. The compiler first recognizes:

- structural recursion on a component of a matched finite recursive value;
- recursion on a constrained integer which moves strictly toward a bound;
- recursion through a finite collection traversal; and
- calls to an already proven terminating function.

When no standard rule succeeds, the complete function may provide
`Decreases ( Measures )` capability evidence after its return type. Each measure
is a pure expression over the inputs into a well-founded order; multiple
expressions form a lexicographic product. Arithmetic, projections, collection
sizes, and statically analyzable pure calls may construct a measure. Every
recursive cycle, including a mutual cycle, must decrease the compatible
measures at each call edge before returning to the same point in the cycle.
The measures are proof information and need not exist at runtime.

The compiler infers `Decreases` evidence when its analysis finds a measure.
An explicit capability guides analysis or publishes the relationship for an
opaque, interface, or higher-order contract. It is never accepted as
trusted-unverified evidence: failure to prove it is a compilation error.

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
- observe `generator-closed` from an abandoned continuation.

A recursive generator call which can occur before any of these boundaries must
decrease a well-founded measure like ordinary recursion. The compiler checks
all decision branches rather than accepting a generator merely because some
path contains `yield`.

Consumers may deliberately traverse an unbounded productive generator. Their
own termination then depends on a statically finite consumer, a stopping
predicate, generator closure, or an external suspension. Materializing an
unbounded generator as a finite container is rejected unless a bound is
established.

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

- the qualified spelling for task-field replacement; and
- whether `with` is the final record-reconstruction keyword.

These are grammar and ergonomics decisions rather than open execution
semantics.
