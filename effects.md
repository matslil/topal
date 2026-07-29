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
effect-polymorphic function, symbolic row parameters. Compiler-inferred
parameters retained in generic intermediate code need no source-level names.
Explicit parameters are named when an opaque contract, foreign boundary,
abstraction requirement, programmer restriction, or otherwise uninferable
relationship needs to state them. Duplicate requirements collapse by identity.
Effect rows have no source ordering; ordering constraints are properties of the
effects and of dependencies between calls.

Effects form dependency evidence rather than a permission system. Read, write,
send, hardware, and other effect declarations specify which access modes
conflict for the same or possibly aliasing resource. The compiler combines
those laws with `DependsOn`, `Independent`, `Conflicts`, `Aliases`, and
`MayAlias` capability evidence to build the execution dependency graph.

Every resource parameter in an effect expression resolves to an existing
identity visible at the function declaration. It may come from a function
input, captured resource, constructed-context member, endpoint, sandbox grant,
or static declaration. An effect name never invents ambient authority:

```topal
load-configuration is fn (
  file : File
) -> Result ( Configuration, ConfigurationErrorCode )
  : Read file
```

Here `file` is the exact resource identity being read. A possible future
standard-library file-system object would be an ordinary visible object; it is
not part of the initial effect model.

## Inference and classification

Calling a function adds its effects to the caller. Constructing or passing a
function value does not perform its effects. A higher-order call adds the
effects of a callback only along paths on which that callback is invoked.

Functions normally rely on inference. A declaration may state an effect upper
bound to document and restrict its implementation. Effect expressions use the
same classification syntax as capabilities but retain their distinct effect
semantics. A bound follows the completed function type:

```topal
copy-file is fn (
  source : File,
  destination : File
) -> Result ( Completed, FileErrorCode )
  : Read source and Write destination
```

The compiler rejects an implementation whose inferred effects exceed that
bound. The bound does not require every listed interaction to occur. A function
with `Effects ()` or only `Read source` satisfies this example; one which also
writes another resource does not. Compiled implementation evidence retains the
smaller exact inferred row.

Effect expressions combine with the same surface operators as capability
expressions:

- `A and B` permits interactions from both effect sets;
- `A or B` retains a statically distinguished alternative effect set;
- nested combinations flatten and duplicate effects collapse by identity; and
- `Effects ()` is the empty runtime effect set.

When an `or` alternative is erased, its safe effect contract is the `and`
combination of every possible alternative. A single implementation whose
runtime branches may perform `A` or `B` likewise has the inferred upper bound
`A and B`; `or` is reserved for retained alternative implementation evidence.

Named and parameterized combinations are ordinary static bindings and
functions:

```topal
FileUpdate is fn static (
  file : File
) -> Effects
  Read file and Write file

CopyEffects is fn static (
  source : File,
  destination : File
) -> Effects
  Read source and Write destination
```

These construct effect expressions; they do not perform the interactions.

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
Capturing the complete function object also captures its exact symbolic effect
evidence. Explicit source-level row variables are therefore unnecessary for
visible generic code. Conceptually:

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

When a higher-order declaration needs an explicit restriction, it classifies
the function input directly:

```topal
read-with is fn (
  file : File,
  reader : ( F : Read file )
) -> Result ( Bytes, FileErrorCode )
  reader file
```

Because effect expressions classify function objects, `F : Read file` also
establishes `F : Function`; spelling `F : Function : Read file` is valid but
redundant.

Ordered overloads may specialize on those effect classifiers:

```topal
process is fn (
  resource : Resource,
  operation : ( F : Effects () )
) -> Completed
  process-without-interaction operation

process is fn (
  resource : Resource,
  operation : ( F : Read resource )
) -> Completed
  process-read-operation ( resource, operation )

process is fn (
  resource : Resource,
  operation : Function
) -> Completed
  process-conservatively ( resource, operation )
```

Declarations are tested in source order. An effect-free function also satisfies
the `Read resource` upper bound, so the narrower empty-effect case appears
first. The final `Function` overload is the conservative fallback.

The compiler normally infers effect-row and resource-identity parameters and
retains their relationships in typed generic intermediate code distributed
with the compiled artifact. Repeated captured identities express exact
equality. At final application compilation, concrete types, callback
implementations, resource identities, capability evidence, and effect evidence
are substituted into that code. The resulting specialization receives the
precise effects derivable for that use.

For example, a generic implementation which invokes `operation resource`
retains the symbolic relationship that its effects include the effects of that
invocation on the selected `resource`. It does not need to invent and expose a
source-level effect-row parameter merely because the callback is not concrete
when the library is first compiled.

An explicit `DependsOn`, `Independent`, `Conflicts`, `Aliases`, or `MayAlias`
classification is needed only when a generic relationship cannot be recovered
from value flow and effect declarations. When neither exact aliasing nor
independence can be established, the generic contract preserves `MayAlias`.

Fallibility remains in `Result`, not in an effect row. A callback returning
`Result ( B, Errors )` gives `map` a fallible value transformation, while a
callback which performs I/O but cannot report failure has an effect without a
`Result`. Neither property silently implies the other.

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

## Remaining surface details

Effect classification, combination, kind implication, and specialization reuse
the settled capability-style syntax. Ordinary visible generic bodies do not
require explicit row-variable declarations.

The exact source declaration for a sandbox adapter and its granted resources
remains part of the foreign-boundary grammar. Private inferred effects may
eventually receive an optional display abbreviation. Compiler diagnostics and
static introspection should display the full semantic row regardless of any
shorthand.
