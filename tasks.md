# Tasks and intrinsic messaging

Every runtime computation in Topal executes in a task. A task owns private
state and receives interactions through functions declared directly in its
task context. Task interaction is intrinsic to the language: programs do not
expose queues, mutexes, global variables, or other synchronization mechanisms.

## Task specialization and definitions

`Task` accepts an option record and constructs a specialized task type. The
implementation is a value of that type:

```topal
Counter is Task (
  queue-size is 10,
  identity is counter
)

counter-service is Counter
  count : Nat

  start is fn ( initial : Nat ) -> Completed
    @ count is initial
    Completed

  increment is fn (
    _ : MessageContext,
    amount : Nat
  ) -> Unit
    @ count is @ count + amount

  current is fn (
    _ : MessageContext,
    Unit
  ) -> Result ( Nat, () )
    @ count
```

Task fields are always accessed through the current-context `@` qualifier inside
handlers. A left side such as `@ count is value` replaces persistent task state
after validating the complete new value. An unqualified `count is value`
instead declares an immutable local binding and never updates task state.

The option record contains task-wide configuration rather than its implemented
interfaces. A task definition establishes its interfaces from the functions
and generators it actually implements, so separate definitions of the same
specialized task type may implement different interfaces.

Applying a task definition supplies the parameters of `start` and constructs a
task instance:

```topal
counter is counter-service 0
counter increment 2
value : Nat is counter current Unit
```

The value `counter` is a typed capability, not a reference to the task's state
or queue. It permits only the interactions declared by `Counter`. Task identity
and messaging authority remain distinct; possessing a task identifier does not
grant permission to send it arbitrary messages.

Functions called by a handler normally execute in the same task. Only a call
through another task capability crosses a task boundary. Library functions
therefore remain ordinary reusable functions and do not need to be declared as
tasks.

## Task identity and namespaces

The `identity` option introduces the stable identity component of the
specialized task type in the ordinary Topal namespace where that type is
declared. Each definition of the specialized type adds its own declaration
identity. Multiple runtime instances of one definition remain distinct even
though they share that stable definition identity.

Task identity is structured namespace identity, not a filesystem path or a
string assembled with punctuation. Ordinary namespace selection retains
Topal's space-separated form. When tasks span networked nodes, a dynamically
constructed node namespace occupies the first stage and the remaining stages
are ordinary Topal namespacing:

```topal
node services search indexer
```

The representation remains opaque, but equality, namespace narrowing, and
diagnostic formatting operate on the structured identity. Identity is
descriptive and usable for discovery; it does not itself grant messaging
authority.

## Shared function interfaces

[`Interface`](interfaces.md) declarations group function and generator shapes
without choosing an implementation mechanism. A source context may implement
such an interface with ordinary functions and generators, while a task may
implement the same interface with events, requests, and streams.

An interface which permits a request or stream implementation across a task
boundary declares its final return through `Result`; the task boundary can
fail even when a direct implementation cannot. Applying the interface to a
task or endpoint produces implementation evidence
which records the selected handler, transport, effects, ordering, admission,
termination, and generator-closure behavior. Calling code can consequently
retain the same operation interface while the compiler selects direct calls or
message passing and preserves the implementation's dependency and optimization
information.

Every task has an implicit concrete interface consisting of its published
message handlers. Explicit interfaces provide restricted endpoint views. An
endpoint grants authority only for the operations in its view; task identity or
descriptive names do not grant arbitrary message authority.

## Message context

Every dispatched task handler receives the compiler-provided `MessageContext`
as its first input:

```topal
MessageContext is Record
  session-id : SessionId
  sender : Endpoint
```

The session identifies the interaction through which the handler was invoked.
The sender is the endpoint of the sending task associated with that
interaction. Both are message-implementation information and consequently do
not appear in an implementation-independent `Interface`.

Bundling this information into one language-defined record leaves room for
later language versions to add context fields without adding more distinguished
handler inputs. A handler which discards the record or selects only existing
fields retains the same source declaration; compiled representations carry the
language-version information needed to diagnose or adapt incompatible binaries.

When a handler implements an interface operation, the compiler checks
conformance after removing the leading `MessageContext` from the handler shape.
For example:

```topal
CommandProcessor is Interface
  process-cmd is fn (
    cmd : String
  ) -> Result ( String, CommandErrorCode )

command-service is CommandTask
  CommandProcessor
    process-cmd is fn (
      message : MessageContext,
      cmd : String
    ) -> Result ( String, CommandErrorCode )
      record-session message session-id
      process cmd
```

This is a specific message-implementation adaptation, not general function
subtyping. A direct implementation of `CommandProcessor` receives only `cmd`.
A task handler, including one belonging only to a task's implicit interface,
always declares the leading context. Generator handlers follow the same rule
for their initial input. The distinguished context slot does not count against
the interface operation's zero-to-two ordinary operands.

When the handler does not use the context, `_` explicitly discards its name
without discarding its type:

```topal
process-cmd is fn (
  _ : MessageContext,
  cmd : String
) -> Result ( String, CommandErrorCode )
  process cmd
```

`_` introduces no binding and may not be referenced. It is only the reserved
spelling for an intentionally unnamed identifier; it is not a wildcard and has
no independent pattern-matching meaning.

## Service discovery

When the compiler detects a task definition, it records the definition's
identity, endpoint construction, and every explicitly implemented
[`Interface`](interfaces.md). It emits the corresponding registration with a
compiler-created service broker. Static compilation records definitions;
runtime registration supplies the endpoint for each live task instance.
Registration becomes discoverable according to the task's startup contract and
is withdrawn when termination begins.

Every task handler can select `service-broker`, a compiler-provided endpoint
implementing `ServiceBrokerInterface`. The interface is public so an
application or execution environment may supply an alternative broker task
without changing clients:

```topal
ServiceBrokerInterface is Interface
  find-task is fn (
    identity : TaskIdentity
  ) -> Result ( Endpoint, ServiceBrokerErrorCode )

  find-interface is generator (
    interface : Interface,
    within : Namespace
  )
    yields Endpoint
    resumes Unit
    -> Result ( Unit, ServiceBrokerErrorCode )
```

The `within` namespace narrows where interface discovery starts. Omitting that
narrowing searches the namespaces accessible through the selected broker; the
exact optional-operand spelling remains provisional. Namespace search respects
ordinary visibility and does not traverse inaccessible declarations.

`find-task` selects an exact identity. `find-interface` may produce any number
of live endpoints whose definitions explicitly implement the requested
interface. The broker returns endpoints with the corresponding implementation
evidence, allowing the requested interface to construct a restricted callable
view. A general endpoint or task identity never grants calls outside such a
view.

The broker database records implementations and live instances, not specialized
`Task` declarations. Those remain ordinary type declarations in their defining
namespaces. Interface conformance likewise belongs to each task definition
rather than to the specialized task type.

## Interaction inference

The declaration determines the message interaction without separate `event`,
`request`, or `stream` directives:

```text
fn (...) -> Unit               event without a completion reply
fn (...) -> Result ( Completed, ApplicationErrors )   completion request
fn (...) -> Result ( Value, ApplicationErrors )       value request
generator ... -> Result ( Value, ApplicationErrors )  stream
```

An event call may still account for declared queue placement, backpressure, or
task lifetime behavior, but it does not wait for the handler to finish. A
`Completed` response is a distinguished zero-data completion value rather than
a synchronization object. Receiving it proves that the handler finished and
establishes the corresponding ordering dependency.

Every ordinary function handler which has a response channel returns `Result`.
Consequently, plain `Completed` and plain value results are invalid handler
shapes. `Result ( Unit, ApplicationErrors )` is also invalid: `Unit` declares
an event with no completion response, while `Result` would require such a
response to report success or failure. A handler which returns only completion
evidence uses `Result ( Completed, ApplicationErrors )`.

These restrictions apply to published message handlers, including handlers in
a task's implicit interface. They do not constrain private helper functions or
the language-defined lifecycle handlers. An explicit interface operation
returning plain `Completed` or another plain value can have a direct
implementation, but cannot be implemented by task message passing. To permit
both direct and task implementations, the operation declares
`Result ( Completed, ApplicationErrors )` or
`Result ( Value, ApplicationErrors )`.

The same `Unit` and `Completed` distinction applies to a direct implementation
of a shared interface. `Unit` establishes no completion dependency and permits
concurrent execution when values, effects, and scope allow it. `Completed`
establishes that execution finished before a dependent continuation proceeds;
it does not require blocking an operating-system thread.

A generator handler establishes a stream. Its yielded type is delivered from
the serving task, its resume type is delivered back to that task, and its final
return terminates the stream. The final return of an ordinary task generator
handler must be a `Result`, including
`Result ( Unit, ApplicationErrors )` when the stream has no application
summary. Individual yields are not wrapped. A generator resumed with `Unit` is
a one-way server-to-caller stream.

Task messaging adds the language-defined `task-terminated` error code to the
declared application errors of each request or stream. This does not add
another wrapper: `Result ( Value, ApplicationErrors )` remains the operation's
declared result type, and its effective error-code set is the union of the
declared codes and `TaskErrorCode`. Ordinary error matching, projection, and
propagation apply.

An endpoint denotes a task instance which existed in the application. Task
messaging therefore does not add separate unavailable, admission, or transport
codes. Queue limits and admission policy are interaction behavior rather than
portable errors. All endpoints are local to the application; communication
with an external or remote service is implemented by a local mirror task. The
mirror reports failures through the application error codes declared by its
interface, or terminates and consequently produces `task-terminated` for its
pending interactions.

The additional code belongs to the selected implementation evidence. A
direct implementation retains only its declared error codes; selecting a task
implementation exposes the wider effective set. If the implementation is
erased or chosen dynamically, the compiler uses the union required by all
remaining alternatives.

## Isolation and suspension

Task state is private and cannot escape through a message. Only one executing
handler segment has authority over a task's state. When a handler suspends on a
request or yields from a stream, that segment releases its state authority and
the task may handle another message. On resumption it observes current task
state rather than retaining a hidden mutable view from before suspension.

Values deliberately retained across suspension are immutable snapshots. Code
which requires state not to have changed must express that requirement through
version evidence, a transaction, or a protocol dependency rather than by
holding an invisible lock.

The compiler derives ordering and communication dependencies from task calls.
It may implement an interaction as a direct call, queue operation, state
machine, or other mechanism when the choice preserves the declared event,
completion, failure, ordering, and isolation behavior.

## Task construction and lifecycle

`start` is the task's lifecycle constructor. It is not callable through the
task capability and does not appear in its message protocol. The compiler and
runtime allocate the hidden task identity, messaging infrastructure, and task
scope before invoking it. Its parameters are consequently the parameters used
to construct the task. Every task definition must provide `start`; lifecycle
handlers do not form a mandatory `TaskInterface`.

The result of `start` determines whether construction waits for initialization:

```text
start : Input -> Unit
Task Input -> Task
```

The task capability is returned without waiting for `start` to finish.

```text
start : Input -> Completed
Task Input -> Task
```

The task capability is returned only after `start` finishes.

```text
start : Input -> Result ( Completed, StartupErrors )
Task Input -> Result ( Task, StartupErrors )
```

Construction waits and returns either the initialized task or its startup
error. `start -> Result ( Unit, StartupErrors )` is invalid for the same reason
as it is for an ordinary task handler: a non-waiting interaction has no
response channel on which to report the error.

Messages sent to a task during non-waiting startup may enter its queue, but
ordinary handlers do not run until `start` finishes. Every successful path
through `start` must establish all task state. A non-waiting `start -> Unit`
must handle any internal failure and still establish valid state because its
creator has no startup-result channel.

An ordinary task may also define the platform-independent `terminate`
lifecycle handler:

```topal
terminate is fn (
  reason : TerminationReason
) -> Unit
  finish-pending-work reason
```

`terminate` is not callable through an endpoint. If omitted, it behaves as a
no-op returning `Unit`; normal child-scope and resource cleanup still occurs.
All non-root tasks use `Unit`, keeping termination independent of the concrete
task type when an endpoint was discovered by interface.

The task may invoke `terminate` itself. Its owning instantiated `Task` value may
also invoke it, and the enclosing task scope invokes it when the instance
leaves scope. A discovered `Endpoint` does not acquire this ownership authority.
A self-call uses `terminate reason`; an owner uses `task terminate reason`.
A lifecycle termination is queued and selected in the task's ordinary message
order. Messages ahead of it run first; messages which have not begun when it is
selected receive `task-terminated`, or are discarded when they are `Unit`
events.

A task can expose its own policy-controlled hard-stop message:

```topal
stop is fn (
  _ : MessageContext,
  reason : StopReason
) -> Unit
  terminate reason
```

Self-termination requests the lifecycle transition rather than recursively
calling the lifecycle handler. The call is terminal in the current handler.
Admission of ordinary work stops, the lifecycle `terminate` handler runs
exactly once, and task-owned resources are automatically cleaned up when it
returns. Suspended ordinary handlers are invalidated without executing more
programmer code; their continuation-owned resources receive automatic cleanup.
Every suspension point must therefore leave both task state and continuation
state well-defined and cleanly destructible. Concurrent termination requests
join the transition already in progress rather than running the lifecycle
handler again.

When suspended handlers should instead finish, an ordinary message handler may
end with the language-defined terminal operation `terminate-cleanly reason`:

```topal
stop is fn (
  _ : MessageContext,
  reason : TerminationReason
) -> Result ( Completed, TerminationErrorCode )
  terminate-cleanly reason
```

`terminate-cleanly` atomically stops admission. Queued and new requests receive
`task-terminated`, and queued and new `Unit` events are discarded. Handler
continuations which were already suspended may resume and finish normally, but
no new ordinary handler begins. Once those continuations finish, the lifecycle
`terminate` handler runs and automatic cleanup follows. The clean transition
does not silently fall back to hard termination if a continuation fails to
finish.

No source code can execute after `terminate-cleanly`. The compiler permits it
only as the terminal expression of a message handler returning `Unit` or
`Result ( Completed, TerminationErrorCode )`. With `Unit`, the sender does not
observe completion. With `Result ( Completed, TerminationErrorCode )`, the
runtime retains the initiating interaction session and replies only after the
lifecycle handler and cleanup finish. Termination or cleanup failure is
returned instead of `Completed`. Other request handlers cannot use the
operation.

Termination may instead be observed as `task-terminated` in the existing
`Result` of a pending request or the final return of a stream, the result of
awaiting an owned child, or an explicit broker monitor/join operation. The
error uses Topal's stable task error domain. Its structured error information
retains the identity, reason, and underlying failure; these do not create more
portable task error codes.

A reply or final stream result becomes committed when the task runtime accepts
it as the terminal result of that interaction. A committed result wins over
concurrent termination even when the caller has not yet observed it. If
termination commits first, the interaction returns `task-terminated`. Values
already yielded by a stream remain observed, but termination before its final
result commits makes that final result `task-terminated`.

The exact surface operation for non-owning monitoring remains provisional, and
must atomically install the monitor so termination cannot race with lookup.

## The application root task

`application.t` is itself the application's root task context. It does not
declare a redundant named application type or publish a conventional `main`
function. Topal creates the root task and invokes its `start` handler with the
application arguments and environment variables:

```topal
# application.t
use lang topal-unix

configuration : Configuration
server : Server

start is fn (
  arguments : CommandArguments,
  environment : Map ( String, String )
) -> Result ( Completed, ApplicationErrorCodes )

  configuration is load-configuration ( arguments, environment )
  server is Server configuration
  Completed

signal is fn (
  _ : MessageContext,
  signal : UnixSignal
) -> Unit
  handle-signal signal

terminate is fn ( reason : TerminationReason ) -> ExitStatus
  server stop reason
  success
```

Returning from `start` completes root-task initialization; it does not terminate
the application. The application continues receiving platform and task
messages until its root scope terminates according to the selected lifecycle.
Unlike ordinary tasks, the root task's `terminate` result is determined by the
selected platform and becomes the application's return value. A platform may
require `Unit`, an exit status, or another platform-defined result.

The selected Topal language features define the application protocol. A Unix
feature may provide command arguments, environment variables, signals, and
orderly shutdown, while an Android feature may provide its platform lifecycle
and application events.
The feature specifies recognized handler names, their types, ordering, and
delivery guarantees. Other functions in `application.t` remain private helper
functions executed by the root task.

Operating-system and framework adapters hold restricted capabilities for this
application protocol; they do not gain access to arbitrary application
functions or state. Events arriving during startup are queued until `start`
finishes, so platform handlers cannot observe partially initialized state.

Environment variables are supplied as a map at the same time as command
arguments. The map contains the environment provided by the platform for this
application execution; it is an ordinary immutable value rather than access to
process-global state.

Starting a child process always requires an explicit environment-variable map.
There is no operation which implicitly inherits the parent process environment.
An application which wants inheritance passes the environment received by
`start`, with any intended changes represented in the map supplied to the
child. An empty map starts the child with no environment variables.

## Runtime scope

All runtime Topal code has a current task identity and task scope. Ordinary
function calls inherit the caller's task, child tasks belong to structured
task scopes, and foreign callbacks enter Topal by delivering a typed task
interaction. Static evaluation constructs compile-time objects and is not
runtime execution, so it does not require a runtime task.

## Structured child scopes

Creating a child task requires a current task scope. The child belongs to that
scope even when its restricted messaging capability is passed elsewhere. A
scope does not complete until all children have completed, their results have
been accounted for, and their resources have been destroyed.

Leaving a successful scope requests orderly completion of unfinished children.
Leaving because of failure queues hard termination for the remaining children
before cleanup finishes. The scope waits for lifecycle handlers and
destructors; it does not detach computation which can access scoped resources.

If a child fails before its result is awaited, the scope retains that failure.
The first failure in deterministic dependency order is primary, terminates
dependent siblings, and records later failures as contextual causes.
Independent children may finish concurrently, but scheduler timing does not
select an observably different primary error.

The semantic operations are:

```text
task-scope body
start-child Task input
await child
```

Their exact surface spelling remains provisional. `await` consumes a one-time
completion obligation; it does not expose a general mutable future object.

## Termination and generator closure

Topal exposes no general cancellation operation or asynchronous exception.
Ownership-driven hard termination uses the queued lifecycle transition above.
Application-defined graceful shutdown uses an ordinary protocol handler ending
in `terminate-cleanly`.

Dropping a stream or linear generator continuation does not expose an explicit
cancellation operation. It resumes the serving generator's suspended `yield`
with `generator-closed`. The generator may perform deliberate shutdown work,
then returns and enters automatic cleanup. It cannot yield again after
observing the close signal. The owning scope waits for shutdown and cleanup and
retains their failures.

A successfully admitted non-stream request is a mandatory reply interaction.
The requester remains suspended until the reply commits or the target returns
`task-terminated`; there is no first-class pending request which can be dropped.
`with-timeout` is the deliberate exception described below: it lets one handler
stop observing the reply while the runtime retains and eventually discards that
reply through the hidden session.

## Waiting for alternatives

A scope may wait for the first acceptable result among several interactions.
The operation consumes a finite labeled product of pending interactions and
returns a union identifying the selected alternative. When several alternatives
are already available at the same logical point, declaration order is the
deterministic tie breaker.

Non-selected interactions remain owned by the scope and must produce their
replies; selection never silently detaches an interaction. The runtime may
retain a reply on behalf of a handler which has explicitly stopped observing it
through `with-timeout`.

## Short delays and message timeouts

A standard-library short-delay operation may pause the currently running task,
which is useful for bounded waits between hardware readiness checks. It uses a
relative monotonic time quantity and records the clock or hardware timing
effect. Other handlers of that task do not run during the pause. The exact
library function name and target-specific useful duration limits belong to the
standard-library and platform design.

Ordinary message timeouts use message passing instead:

```topal
5[s] with-timeout ( network request packet )
```

The left operand is a nonnegative relative monotonic time quantity. The
parenthesized right operand must be one reply-bearing request or stream message
interface call. A `Unit` event has no reply to wait for and is invalid here.

For a request, evaluation initiates the message call and registers one timeout
with a compiler-created application-local timeout server. For a stream, the
same duration applies independently whenever its consumer waits for the first
yield or final return and whenever it resumes the stream to await the next yield
or final return.

At the beginning of each wait interval, the runtime converts the relative
quantity to a hidden absolute monotonic deadline before sending the
registration. Queueing and delivery latency therefore do not extend that
interval. The source language exposes neither the absolute value nor the clock
representation.

The expression combines the message call's effects with its timeout-server and
monotonic-clock interactions. Those compiler-provided identities are recorded
in implementation evidence without exposing ambient file, clock, or service
authority to ordinary functions.

Each timeout registration carries the requesting handler's mandatory
`SessionId`, the absolute deadline, and a fresh compiler-generated timeout ID.
The calling handler suspends awaiting either the corresponding message result
or a timeout event sent back by the server. The timeout ID distinguishes
sequential or concurrent registrations belonging to the same handler and is
never a source-level value.

When a request reply, stream yield, or stream final return wins its interval,
the runtime atomically marks that timeout ID cancelled and sends a cancellation
event to the timeout server. Once this local cancellation completes, no event
for that timeout can be observed by the handler. A timeout event already queued
or in transit is discarded by the runtime. When the timeout wins, duplicate
timeout events cannot resume the handler again.

A stream has no active timer while its yielded value is being handled or while
its continuation is retained without being resumed. Resuming it begins a new
interval with a new absolute deadline and timeout ID. Consumer processing time
therefore cannot cause a provider-side timeout.

For a request, the timeout does not cancel the executed message operation. Its
eventual reply is accepted and discarded through the retained hidden request
session. For a stream, timeout ends the consumer's stream with an error and
abandons its continuation; the serving generator follows the existing
`generator-closed` path. Values already yielded remain observed. Neither form
hard-terminates the serving task. A protocol which can stop the underlying
network, hardware, or service operation defines that separately.

If the message call has effective type:

```topal
Result (
  Response,
  (
    NetworkErrorCode,
    TaskErrorCode
  )
)
```

the complete timeout expression has:

```topal
Result (
  Response,
  (
    NetworkErrorCode,
    TaskErrorCode,
    TimeoutErrorCode
  )
)
```

The caller-side failure has qualified code
`timeout-error timeout-occurred` and domain `lang with-timeout`. A handler may
return the same qualified code under its own domain; domain matching
distinguishes the reporting boundary. If the handler already declares
`TimeoutErrorCode`, the duplicate vocabulary collapses without collapsing
domains.

For a stream declaration:

```topal
receive-packets is generator (
  request : PacketRequest
)
  yields Packet
  resumes Unit
  -> Result ( TransferSummary, NetworkErrorCode )
```

`with-timeout` preserves `yields Packet` and `resumes Unit`, while its final
message result becomes:

```topal
Result (
  TransferSummary,
  (
    NetworkErrorCode,
    TaskErrorCode,
    TimeoutErrorCode
  )
)
```

Individual yields are not wrapped. If an interval times out, the stream ends
with the caller-side timeout error as its final result.

A request reply, stream yield, or stream final return committed before its
absolute interval deadline wins. Once the deadline is reached without a
committed result for that interval, the timeout may win. Hidden sessions and
timeout IDs ensure that a result, timeout, cancellation, or duplicate arriving
after selection cannot resume the handler a second time.

## Backpressure and queue bounds

Every event or request protocol declares its admission behavior: bounded wait,
bounded rejection with `Result`, or contained loss for an isolated diagnostic
event. An ordinary `Unit` event may not silently discard a message unless its
protocol explicitly has isolated diagnostic semantics.

Queue capacity is an implementation choice only within the declared admission
behavior. A sender which may suspend or fail because of backpressure exposes
that interaction in its protocol and effect contract, allowing queue
dependencies to participate in deadlock checking.
