# Effects and observable interaction

An effect describes an observable interaction performed by a function. It is
part of the function's compiled type alongside its inputs, output, staticness,
constructed-context requirements, and protocol communication. Effects constrain
ordering and parallel execution without replacing explicit `Result` control
flow.

The compiler infers effects through ordinary calls and message interactions.
A source `Interface` describes function and generator shapes without acquiring
the effects of one implementation. Applying that interface to a concrete
context, packaged implementation, task, or endpoint produces implementation
evidence containing its inferred effect row. Published compiled functions and
implementations retain that evidence, so their callers can check dependencies
and optimize without requiring effects to be repeated in the source interface.

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

Effects form dependency evidence rather than a permission system. Read, write,
send, hardware, and other effect declarations specify which access modes
conflict for the same or possibly aliasing resource. The compiler combines
those laws with `DependsOn`, `Independent`, `Conflicts`, `Aliases`, and
`MayAlias` capability evidence to build the execution dependency graph.

## Inference and declarations

Calling a function adds its effects to the caller. Constructing or passing an
function value does not perform its effects. A higher-order call adds the
effects of a callback only along paths on which that callback is invoked.

Functions normally rely on inference. A declaration may state an effect upper
bound to document and restrict its implementation. The compiler rejects an
implementation whose inferred effects exceed that bound.

Public compiled implementations expose their inferred effect evidence alongside
their callable interface. This evidence belongs to that implementation, not to
the implementation-independent `Interface` type. Changing an implementation
may therefore change which callers accept it and which optimizations are
available without changing the source interface it implements.

Finite dynamic selection retains a tagged sum of implementation identities and
their exact effects when specialization can usefully distinguish the
alternatives. Otherwise the implementation identity is erased. Erasure retains
the intersection of capability guarantees but the union of possible effects;
unknown resource relationships become `MayAlias` and use the safest ordering.
Matching a retained alternative may select scheduling, batching, allocation,
transport, or other optimized code, but every path must preserve the
interface-observable semantics.
An explicit bound remains useful when an API intentionally restricts an open
family of implementations, but dynamic selection does not by itself require
early effect erasure. Foreign interactions are represented by their sandbox
boundary because the compiler cannot infer their internals.

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

The compiler normally infers effect-row and resource-identity parameters and
retains their relationships in compiled generic evidence. Repeated captured
identities express exact equality. An explicit `DependsOn`, `Independent`,
`Conflicts`, `Aliases`, or `MayAlias` classification is needed only when a
generic relationship cannot be recovered from value flow and effect
declarations. When neither exact aliasing nor independence can be established,
the generic contract preserves `MayAlias`.

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
Reusing the same captured identity proves the relationship directly. Distinct
names do not prove distinct resources; without `Independent` or equivalent
evidence the compiler retains `MayAlias` and orders potentially conflicting
interactions conservatively.

Hardware reads and writes are ordered effects unless their layout and device
capabilities explicitly provide weaker laws. Ordinary file and network
operations are likewise observable and are not silently cached or retried.

## Handling and containment

An effect requirement is discharged only at a boundary which supplies an
implementation capability:

- application composition supplies operating-system and service capabilities;
- a task handles the interactions in its declared protocol;
- a constructed context supplies a fixed contained diagnostic capability; or
- a sandbox adapter implements a declared boundary protocol using only granted
  resource capabilities.

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
selected from a context combines the inferred effects of its handlers with its
endpoint and transport effects. The endpoint's compiled implementation evidence
retains this composition even though its source interface also admits a direct
function implementation.

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

Abandoning a linear continuation resumes its suspended `yield` with the
language-defined `generator-closed` error. The generator may perform explicit
shutdown work before returning; it cannot yield again on that path. Resource
cleanup then runs in the current structured task scope. Shutdown and cleanup
effects remain part of the enclosing scope's contract, which waits for them
and retains their failures.

## Sandboxed foreign effects

Initial foreign execution occurs through a sandboxed adapter. Values cross
through validated layouts, copied or serialized representations, declared
message protocols, and explicitly granted resource capabilities. Foreign code
does not receive Topal references, borrowed storage, raw continuations,
arbitrary callbacks, task internals, or unrestricted process memory.

Topal records sends, receives, and granted-resource use at the sandbox boundary.
Unknown internal behavior therefore cannot silently become an unknown effect on
the entire application. If the sandbox can access a file, device, endpoint, or
other external resource, the adapter must receive that capability explicitly
and the corresponding effect participates in the normal dependency graph.

Programmer capability claims may describe additional semantic or optimization
properties, but cannot remove sandbox, validation, ownership, or containment
checks. Future language-specific adapters may provide stronger promises from
their own interface and safety systems; those are not part of the initial
foreign model.

## Surface syntax still to choose

The semantic model leaves these grammar choices open:

- how an explicit effect upper bound follows a function type;
- how effect-row parameters are named in higher-order declarations;
- how a source declaration names a sandbox adapter and its granted resources;
  and
- whether private inferred effects have an optional display abbreviation.

Compiler diagnostics and static introspection should display the full semantic
row regardless of the chosen source shorthand.
