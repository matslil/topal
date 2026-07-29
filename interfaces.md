# Function and message interfaces

An interface describes a related set of function and generator declarations
without choosing how they are implemented. The same interface may be implemented
by ordinary functions and generators in a source context or by handlers reached
through task message passing.

`Interface` is a type constructor. Applying it to declarations constructs an
interface type:

```topal
Lexer is Interface
  warm-up is fn (
    configuration : Configuration
  ) -> Unit

  parse is fn (
    command : String
  ) -> Result ParseResult

  parse-tokens is generator (
    source : String
  )
    yields Token
    resumes Unit
    -> Result ParseResult
```

The declarations specify interaction shapes, not implementation bodies,
locations, effects, scheduling, or representation. A function declaration
specifies its classified inputs and result. A generator additionally specifies
its yield, resume, and final return directions.

## Context implementations

An interface may be applied directly in a package, module, application, library,
ordinary source-file scope, or another constructed context:

```topal
Lexer
  warm-up is fn (
    configuration : Configuration
  ) -> Unit
    initialize configuration

  parse is fn (
    command : String
  ) -> Result ParseResult
    parse-command command

  parse-tokens is generator (
    source : String
  )
    yields Token
    resumes Unit
    -> Result ParseResult

    tokenize source foreach { token }
      Unit is yield token
    parse-command source
```

This construction checks that the declarations in the block collectively
implement `Lexer`. Each required declaration must occur exactly once and match
its declared function or generator shape. Missing, duplicate, and incompatible
declarations are errors at the construction.

The declarations belong directly to the surrounding source context. The
interface construction does not introduce another mandatory namespace or
runtime object. Visibility remains ordinary declaration visibility, so a module
may publish `parse`, keep `warm-up` private, or expose the complete interface
through a library facade without exporting an intermediate implementation
value.

Different contexts may independently implement and publish the same interface.
Conformance is intentional: declarations which merely happen to have matching
names and types do not implement an interface without an explicit construction.

## Packaged implementations

An implementation may be packaged as a named value when code needs to pass,
return, store, select, or compose it:

```topal
local-lexer is Lexer
  warm-up is fn (
    configuration : Configuration
  ) -> Unit
    initialize configuration

  parse is fn (
    command : String
  ) -> Result ParseResult
    parse-command command

  parse-tokens is generator (
    source : String
  )
    yields Token
    resumes Unit
    -> Result ParseResult
    implementation
```

The value retains the selected declarations and their implementation evidence:

```topal
parse-with is fn (
  lexer : Lexer,
  command : String
) -> Result ParseResult
  lexer parse command
```

Packaging is optional. A module which exposes its implementation functions
directly does not need a redundant `local-lexer is Lexer` binding. The compiler
may specialize a statically selected package completely; a runtime dispatch
representation is required only when selection remains dynamic.

Interfaces compose like other constructions. A wrapper can accept one
implementation and construct another:

```topal
tracing-lexer is fn (
  underlying : Lexer,
  logger : Logger
) -> Lexer

  Lexer
    warm-up is fn (
      configuration : Configuration
    ) -> Unit
      logger record "Lexer warm-up"
      underlying warm-up configuration

    parse is fn (
      command : String
    ) -> Result ParseResult
      logger record command
      underlying parse command

    parse-tokens is generator (
      source : String
    )
      yields Token
      resumes Unit
      -> Result ParseResult
      implementation
```

## Message implementations

A task context may implement the same interface with message handlers. The
function and generator shapes then determine the messaging interaction:

```text
fn Input -> Unit               event without a completion reply
fn Input -> Result Completed   completion request
fn Input -> Result Value       value request
generator ... -> Result Value  stream
```

For example, a task can apply `Lexer` around the relevant handler declarations:

```topal
LexerTask is Task (
  queue-size is 32,
  identity is lexer
)

LexerService is LexerTask
  grammar : Grammar

  Lexer
    warm-up is fn (
      _ : MessageContext,
      configuration : Configuration
    ) -> Unit
      grammar is load-grammar configuration

    parse is fn (
      _ : MessageContext,
      command : String
    ) -> Result ParseResult
      grammar parse command

    parse-tokens is generator (
      _ : MessageContext,
      source : String
    )
      yields Token
      resumes Unit
      -> Result ParseResult
      implementation
```

The endpoint implements the interface even though a call crosses a task
boundary. The call retains the same source operation shape; the selected
implementation evidence tells the compiler whether to use an ordinary call,
generator continuation, event, request, or stream.

The leading `MessageContext` is mandatory for message handlers but is projected
away when checking conformance with the implementation-independent interface.
It supplies the session identity and sender endpoint without adding transport
details to `Lexer`. `_` explicitly leaves that context unnamed when the handler
does not use it; it is a discard identifier rather than a pattern wildcard.

`Unit` deliberately provides no completion dependency in either model.
`Completed` is the zero-data evidence that execution finished. It orders
dependent continuation without requiring an operating-system thread to block.
Every ordinary message handler with a response channel must return `Result`.
Plain `Completed` and plain value results are therefore invalid message
implementations. `Result Unit` is invalid for a function interaction with no
completion channel; a zero-data completion uses `Result Completed`. A task
generator's final return must likewise be a `Result`, although `Result Unit`
is valid there because a stream has a final response channel.

An interface operation returning plain `Completed` or another plain value may
have a direct implementation, but cannot have a message implementation. An
operation intended for either mechanism declares `Result Completed` or
`Result Value`. A task implementation adds the language-defined
`task-terminated` code to the declared application codes in the effective
`Result` contract; it does not change the success type or introduce another
result wrapper.

A task's complete set of published handlers forms its implicit concrete
interface. Explicit `Interface` constructions let the task publish restricted
views, and an endpoint carrying one view grants authority only for the
operations in that view. Task names and identities do not grant messaging
authority.

## Implementation evidence

The source interface remains independent of every implementation. When the
compiler applies it to a concrete context, packaged value, task, or endpoint,
it constructs implementation evidence containing everything known about the
selected declarations:

- inferred effects and the resource identities they touch;
- completion, suspension, ordering, and dependency behavior;
- compiler-verified semantic properties;
- optimization facts and non-semantic operational hints; and
- for endpoints, admission, delivery, termination, transport, and handler
  behavior, including the `task-terminated` error code.

This evidence belongs to the interface implementation rather than changing the
interface type. Two implementations of `Lexer` can therefore have different
effects and optimization opportunities while preserving the same source
interaction shapes.

Calls retain the precise implementation evidence for as long as the selected
implementation remains statically known. Generic code can bind that identity
and be specialized using its evidence. Deliberately erasing or dynamically
selecting an implementation retains only facts proved for every remaining
alternative.

Effects are correctness information as well as optimization input. They
prevent unsafe reordering and expose dependencies even though programmers
normally do not annotate them. Optimization hints may be ignored without
changing behavior. Properties which authorize semantic transformations, such
as idempotence or commutativity, require compiler proof or explicit trusted
evidence rather than inference from a declaration name.

Compiled libraries publish implementation evidence with the functions or
packaged implementations they expose. A message endpoint combines the handler
evidence with transport and endpoint evidence. Consequently, separately
compiled and message-based implementations preserve the same analysis model as
direct functions; crossing a task boundary adds communication behavior without
replacing the underlying interface.

An exported generic implementation normally publishes typed generic
intermediate code together with symbolic implementation evidence. A consuming
application can substitute its concrete types, callbacks, resource identities,
effects, and capabilities before producing machine code. Statically known uses
therefore retain precise evidence across a compiled-library boundary rather
than collapsing immediately to a conservative shared implementation.

A library may deliberately publish an opaque shared implementation instead.
It must then provide a sufficient compiled contract for correctness, and its
callers retain only that contract rather than body-derived specialization
evidence. Foreign exports likewise require a concrete ABI and cannot leave
their externally visible representation unresolved.

Foreign or independently deployed implementations must provide trusted,
authenticated, or runtime-enforced metadata before the compiler may use a
property for correctness. Unverified metadata may influence performance only
when ignoring it preserves behavior.

## Relationship to capabilities

An interface specifies callable function and generator shapes. A capability may
add associated objects and semantic laws to such operations. Satisfying an
interface therefore proves that its declarations are implemented; it does not
by itself assert algebraic laws such as equality, ordering, associativity, or
losslessness.

Capabilities remain appropriate when generic code requires those laws.
Interfaces remain appropriate when code needs one implementation of a related
call surface which may be local, packaged, wrapped, or reached through message
passing.
