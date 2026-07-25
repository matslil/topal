# Effects and observable interaction

An effect describes an observable interaction performed by a function. It is
part of the function's compiled type alongside its inputs, output, staticness,
constructed-context requirements, and protocol communication. Effects constrain
ordering and parallel execution without replacing explicit `Result` control
flow.

The compiler infers effects through ordinary calls. Public interfaces record
the inferred set or an explicitly declared upper bound, so changing a public
function to perform a new effect is an interface change.

## Effect identities

Effects are first-class static objects with stable declaration identity. An
effect may be parameterized by a resource identity:

```text
Read file-system
Write file-system
Read clock
Send UserService
HardwareAccess device
```

Two effects with equal descriptive names are not equal unless their declaration
identities and parameters are equal. A resource parameter lets the compiler
distinguish independent interactions, such as writes to separate devices,
without assuming that all operations of one broad category interfere.

A function's effect row is a finite set of effect requirements plus, for an
effect-polymorphic function, named row parameters. Duplicate requirements
collapse by identity. Effect rows have no source ordering; ordering constraints
are properties of the effects and of dependencies between calls.

## Inference and declarations

Calling a function adds its effects to the caller. Constructing or passing an
function value does not perform its effects. A higher-order call adds the
effects of a callback only along paths on which that callback is invoked.

Private functions normally rely on inference. A declaration may state an
effect upper bound to document and restrict its implementation. The compiler
rejects an implementation whose inferred effects exceed that bound.

Public functions always expose a stable compiled effect contract. Source may
require explicit effect declarations for public interfaces even when the
compiler could infer them; the final surface rule remains to be selected.

Static functions have an empty runtime effect row. Constraint predicates,
match guards, equality, ordering, and law proofs are also pure and total.

## Effect polymorphism

Higher-order functions quantify over the effects of their function inputs.
Conceptually:

```text
map : forall effects E.
      List A, ( A -> B with E ) -> List B with E

foreach : forall effects E.
          Sequence A, ( A -> Unit with E ) -> Unit with E
```

The higher-order implementation may add its own effects in addition to `E`.
It may also constrain `E`; parallel traversal, for example, requires evidence
that callback effects are independent between entries or otherwise safely
ordered.

Fallibility remains in `Result`, not in an effect row. A callback returning
`Result B` gives `map` a fallible value transformation, while a callback which
performs I/O but cannot report failure has an effect without a `Result`.
Neither property silently implies the other.

## Ordering and independence

Every effect declaration specifies its ordering law:

- **ordered** interactions with the same resource retain program dependency
  order and cannot be duplicated, removed, or speculated;
- **commutative** interactions may be reordered or combined according to
  declared laws;
- **isolated diagnostic** interactions are not observable by application code
  except through separately declared time or resource-limit interactions; and
- **independent** interactions with provably distinct resource identities may
  execute concurrently.

Data dependency still establishes order even when two effects would otherwise
commute. A function result used by a later call orders the calls.

The compiler may parallelize calls only when value ownership, effect evidence,
and protocol dependencies jointly prove independence. Absence of a shared
effect name is not by itself sufficient when resource identities may alias.

Hardware reads and writes are ordered effects unless their layout and device
capabilities explicitly provide weaker laws. Ordinary file and network
operations are likewise observable and are not silently cached or retried.

## Handling and containment

An effect requirement is discharged only at a boundary which supplies an
implementation capability:

- application composition supplies operating-system and service capabilities;
- a task handles the interactions in its declared protocol;
- a constructed context supplies a fixed contained diagnostic capability; or
- a trusted foreign adapter implements a declared boundary effect.

Handling an effect may translate it into other effects. The handler's contract
records those implementation effects even when clients see only the abstract
effect. Application composition verifies that every reachable requirement is
ultimately handled.

There is initially no unrestricted dynamic effect-handler construct. Ordinary
functions, tasks, protocols, constructed contexts, and application composition provide
the handling boundaries while retaining analyzable control and communication
dependencies.

## Effects and constructed contexts

Selecting a stable immutable context member is pure. Invoking an endpoint
selected from a context performs the communication effects declared by its
protocol.

An isolated diagnostic capability supplied through a context carries a
contained diagnostic effect. Application functions record that dependency,
but the effect cannot return information, publish a capability, change
application state, or alter application control flow. Its implementation may
have private effects which remain inside the containment boundary.

Context selection is therefore not a general mechanism for hiding effects.
Semantically observable service operations use protocols and ordinary effect
contracts even when their endpoint is obtained from a constructed context.

## Effects and generators

Starting or resuming a generator performs effects until its next yield,
suspension, failure, or return. Merely retaining its continuation performs no
effect. The generator type records the effects which any segment may perform.

Abandoning a linear continuation invokes its resource cleanup and cancellation
behavior in the current structured task scope. Cleanup effects remain part of
the enclosing scope's contract.

## Foreign and trusted effects

A foreign declaration must state its value types, layouts, effects, fallibility,
resource ownership, suspension behavior, and callback protocol. The compiler
does not infer these properties from foreign code.

An adapter which claims containment, algebraic laws, or resource independence
that Topal cannot verify is trusted. Trusted declarations are explicit in
source and compiled metadata. Trust applies only to the stated declaration; it
does not create a general unchecked region in surrounding Topal code.

Values crossing the boundary are validated through layouts, serialization, or
declared conversions. Foreign code cannot retain a borrowed Topal value,
resume a continuation twice, call an undeclared callback, or enter an arbitrary
task merely because its ABI can represent an address.

## Surface syntax still to choose

The semantic model leaves these grammar choices open:

- how an explicit effect upper bound follows a function type;
- how effect-row parameters are named in higher-order declarations;
- how a source declaration names a trusted foreign adapter; and
- whether private inferred effects have an optional display abbreviation.

Compiler diagnostics and static introspection should display the full semantic
row regardless of the chosen source shorthand.
