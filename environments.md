# Environments

Topal environments provide the convenient availability of global declarations
without process-global identity or shared mutable state. An environment is a
separate context whose bindings are fixed when an execution is composed. Its
typed declarations are made available explicitly with `use` and selected with
the `@` prefix.

Calls cannot add, replace, or remove environment bindings. Environment lookup
is total and stable: selecting the same declaration in the same environment
always produces the same value. The compiler uses the call tree to find direct
and transitive environment dependencies, not to construct new environments at
runtime.

This model is intended primarily for contained diagnostics such as logging and
tracing, immutable execution context, and stable capabilities such as messaging
service endpoints. It is not a second store for application state.

## Environment declarations

An environment is a declaration context. A declaration exported from that
context is prefixed with `@`:

```topal
logging is environment
  @ log-error is fn ( message : String ) -> Unit
    send-to-log Error message

  @ log-warning is fn ( message : String ) -> Unit
    send-to-log Warning message

  send-to-log is fn ( severity : Severity, message : String ) -> Unit
    contained-logging-implementation ( severity, message )
```

The exact module syntax remains provisional. Conceptually, application code
makes the exported environment declarations lexically available with `use`:

```topal
use application.logging

process-request is fn ( request : Request ) -> Result Response
  @ log-error "Processing request"
  handle-request request
```

`use` does not introduce ordinary unqualified bindings. `@ log-error` and an
ordinary `log-error` remain distinct declarations, and an environment
declaration cannot be accessed without `@`. This makes ambient access visible at
each use while avoiding environment parameters in every algorithm declaration.

The normal algorithm syntax remains in force. In particular, the output type of
an environment algorithm is explicit even when its environment classification
restricts that output to `Unit`.

## Composition and call-tree dependencies

An environment is an immutable mapping from declaration identities to typed
values. Application composition supplies one value for every reachable
environment declaration before execution begins:

```text
application environment
  application.logging.log-error -> contained logger
  services.user-service         -> user endpoint
  services.plugin-events        -> plugin endpoint
```

This mapping remains fixed for the execution. Calls and task scopes neither
derive child environments nor modify the application environment. Deployments
and tests select different implementations by composing different executions.
Two simultaneous roles using the same value type require two distinct
environment declarations.

The call tree determines dependency reachability. If `C` selects an environment
declaration, while `A` calls `B` and `B` calls `C`, then the compiled contracts
for `A` and `B` record the transitive requirement even though neither algorithm
selects the declaration directly. This permits composition checking without
adding an environment parameter to their source declarations.

Closures retain only the fixed environment declarations they use. Structured
child tasks use the same bindings as their parent execution. A task cannot
capture the environment as an unrestricted map or create a modified environment
for its descendants.

## Isolated environment algorithms

An environment algorithm used for diagnostics cannot communicate a result back
to application code. It has an explicit output of `Unit` and cannot propagate an
error:

```topal
@ log-error is fn ( message : String ) -> Unit
  contained-logging-implementation ( Error, message )
```

These declarations are invalid:

```topal
@ log-error is fn ( message : String ) -> Result Unit
  implementation

@ log-status is fn ( Unit ) -> LogStatus
  implementation
```

Failures, buffering, retries, and fallback behavior belong to the contained
logging service. An environment implementation may operate on private state and
external diagnostic sinks, but neither its state nor the result of those
operations is observable by application code. It cannot modify
application-owned values, invoke an application callback, publish a capability,
or let an internal effect or error escape its container.

Returning `Unit` is therefore necessary but not sufficient. The checker must
also reject an implementation whose reachable effects can influence application
state or control flow. Foreign implementations require a trusted adapter which
provides the same containment guarantee.

Diagnostic operations may affect elapsed execution time. Application code can
observe that time only through an independently declared clock, timeout, or
resource-limit interaction. A diagnostic delivery policy must not retry or
suspend without a declared finite bound; nonblocking enqueue with contained
fallback or event loss is the normal implementation.

Exact delivery, externally visible ordering, or failure reporting makes an
operation part of application semantics rather than an isolated diagnostic
operation. Such behavior must use an ordinary effect or messaging protocol.

## Stable environment values

Some environment declarations provide immutable values rather than isolated
algorithms. A messaging endpoint is the primary example:

```topal
UserService is protocol
  GetUser is request UserId -> Result User
  UpdateUser is request UserUpdate -> Result User

@ user-service : Endpoint UserService
```

Selection of the endpoint is pure, total, and stable:

```topal
endpoint is @ user-service
```

For a particular environment `E`:

```text
lookup(E, user-service) = lookup(E, user-service)
```

Both selections identify the same logical endpoint capability. Lookup cannot
fail, consume the endpoint, or choose a different value on a later access. A
missing required binding is an environment-composition error before execution.

The stable logical endpoint need not represent one socket, process, or network
address. Its implementation may perform discovery, pooling, reconnection,
failover, or load balancing while preserving the declared protocol. Those
routing choices are not environment lookup.

## Messaging through endpoints

The protocol, rather than the environment, declares that an algorithm
communicates with a service. The environment selects which endpoint implements
that already-known role:

```topal
use services.users

load-user is fn ( id : UserId ) -> Result User
  @ user-service request GetUser id
```

Conceptually, the algorithm contract records both facts:

```text
communicates UserService
requires environment services.users.user-service
```

Composing another execution can select an in-process test service or a remote
production service. It does not change the algorithm's communication
declaration. Retrieving `@ user-service` remains infallible; the `GetUser`
request may fail or suspend because its protocol explicitly permits that.

Messaging endpoints are capabilities, not references to broker internals or
shared application variables. They permit only the messages declared by their
protocol. Sends, replies, cancellation, ordering, and backpressure occur in
well-defined task scopes, allowing the compiler to derive communication
dependencies without assuming shared mutable state.

Application code should normally import a protocol-specific endpoint rather
than a universal broker:

```topal
@ user-service : Endpoint UserService
```

This is preferred to an endpoint which accepts arbitrary service names and
messages. Composition code may use a broker to construct protocol endpoints,
but application code receives only the resulting restricted capabilities.

## Dependency checking and erasure

Environment declarations do not appear among an algorithm's ordinary inputs,
but the compiler records direct and transitive uses in its compiled type. A
public algorithm can therefore be checked and documented without exposing
environment plumbing in source parameter lists. Application composition must
provide every declaration reachable from its call tree.

An algorithm which neither selects an environment declaration nor calls an
algorithm that does is isolated from the environment. The compiler need not
pass an environment context to it. Stable lookup also permits the compiler to
resolve a value once, specialize a call for a statically known application
environment, or pass an endpoint directly to only the calls which use it.

Static algorithms cannot access runtime environment declarations. Environment
availability and concrete service selection are runtime properties and cannot
participate in compile-time construction of types or other static objects.

## Provisional grammar additions

The `@` symbol is a prefix scope selector rather than part of an identifier. Its
canonical formatting therefore includes a space:

```topal
@ log-error "message"
```

The relevant grammar additions are conceptually:

```ebnf
environment-selection = "@" identifier ;
environment-export    = "@" binding | "@" classification ;
use-declaration       = "use" qualified-identifier ;
```

`environment-selection` is a prefix expression and otherwise follows the normal
application and left-to-right grouping rules. The complete syntax for defining
modules, composing executions, and supplying their stable environment values
remains provisional.
