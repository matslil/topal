# Tasks and intrinsic messaging

Every runtime computation in Topal executes in a task. A task owns private
state and receives interactions through functions declared directly in its
task context. Task interaction is intrinsic to the language: programs do not
expose queues, mutexes, global variables, or other synchronization mechanisms.

## Task declarations

A task groups its state and message handlers:

```topal
Counter is task
  count : Nat

  start is fn ( initial : Nat ) -> Completed
    count is initial
    Completed

  increment is fn ( amount : Nat ) -> Unit
    count is count + amount

  current is fn ( Unit ) -> Nat
    count
```

Applying the task supplies the parameters of `start` and constructs a task
instance:

```topal
counter is Counter 0
counter increment 2
value is counter current Unit
```

The value `counter` is a typed capability, not a reference to the task's state
or queue. It permits only the interactions declared by `Counter`. Task identity
and messaging authority remain distinct; possessing a task identifier does not
grant permission to send it arbitrary messages.

Functions called by a handler normally execute in the same task. Only a call
through another task capability crosses a task boundary. Library functions
therefore remain ordinary reusable functions and do not need to be declared as
tasks.

## Shared function interfaces

[`Interface`](interfaces.md) declarations group function and generator shapes
without choosing an implementation mechanism. A source context may implement
such an interface with ordinary functions and generators, while a task may
implement the same interface with events, requests, and streams.

The interface remains unchanged when an implementation moves across a task
boundary. Applying it to a task or endpoint produces implementation evidence
which records the selected handler, transport, effects, ordering, admission,
and cancellation behavior. Calling code can consequently retain the same
operation interface while the compiler selects direct calls or message passing
and preserves the implementation's dependency and optimization information.

Every task has an implicit concrete interface consisting of its published
message handlers. Explicit interfaces provide restricted endpoint views. An
endpoint grants authority only for the operations in its view; task identity or
descriptive names do not grant arbitrary message authority.

## Interaction inference

The declaration determines the message interaction without separate `event`,
`request`, or `stream` directives:

```text
fn (...) -> Unit               event without a completion reply
fn (...) -> Completed          completion request
fn (...) -> Result Completed   fallible completion request
fn (...) -> Value              value request
fn (...) -> Result Value       fallible value request
generator                     stream
```

An event call may still account for declared queue placement, backpressure, or
task lifetime behavior, but it does not wait for the handler to finish. A
`Completed` response is a distinguished zero-data completion value rather than
a synchronization object. Receiving it proves that the handler finished and
establishes the corresponding ordering dependency.

`Result Unit` is not a valid task-handler result. `Unit` declares that no
completion response exists, while `Result` would require that response to
report success or failure. A fallible handler with no application response uses
`Result Completed`.

The same `Unit` and `Completed` distinction applies to a direct implementation
of a shared interface. `Unit` establishes no completion dependency and permits
concurrent execution when values, effects, and scope allow it. `Completed`
establishes that execution finished before a dependent continuation proceeds;
it does not require blocking an operating-system thread.

A generator handler establishes a stream. Its yielded type is delivered from
the serving task, its resume type is delivered back to that task, and its final
return terminates the stream. A generator resumed with `Unit` is a one-way
server-to-caller stream.

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

## Task construction and `start`

`start` is the task's lifecycle constructor. It is not callable through the
task capability and does not appear in its message protocol. The compiler and
runtime allocate the hidden task identity, messaging infrastructure, and task
scope before invoking it. Its parameters are consequently the parameters used
to construct the task.

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
start : Input -> Result Completed
Task Input -> Result Task
```

Construction waits and returns either the initialized task or its startup
error. `start -> Result Unit` is invalid for the same reason as it is for an
ordinary task handler: a non-waiting interaction has no response channel on
which to report the error.

Messages sent to a task during non-waiting startup may enter its queue, but
ordinary handlers do not run until `start` finishes. Every successful path
through `start` must establish all task state. A non-waiting `start -> Unit`
must handle any internal failure and still establish valid state because its
creator has no startup-result channel.

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
) -> Result Completed

  configuration is load-configuration ( arguments, environment )
  server is Server configuration
  Completed

signal is fn ( signal : UnixSignal ) -> Unit
  handle-signal signal

stop is fn ( reason : StopReason ) -> Completed
  server stop reason
  Completed
```

Returning from `start` completes root-task initialization; it does not terminate
the application. The application continues receiving platform and task
messages until its root scope terminates according to the selected lifecycle.

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
Leaving because of failure or explicit cancellation cancels the remaining
children before cleanup finishes. The scope waits for cancellation handlers
and destructors; it does not detach computation which can access scoped
resources.

If a child fails before its result is awaited, the scope retains that failure.
The first failure in deterministic dependency order is primary, cancels
dependent siblings, and records later failures as contextual causes.
Independent children may finish concurrently, but scheduler timing does not
select an observably different primary error.

The semantic operations are:

```text
task-scope body
start-child Task input
await child
cancel child reason
```

Their exact surface spelling remains provisional. `await` consumes a one-time
completion obligation; it does not expose a general mutable future object.

## Cancellation

Cancellation is a typed interaction from a task scope, not an asynchronous
exception injected at an arbitrary instruction. A task observes it at a
suspension, protocol operation, generator boundary, or explicit cancellation
check. Between such points, termination or productivity still requires finite
computation.

Cancellation begins cleanup for the cancelled branch. Destructors, protocol
termination, and cancellation handlers run in dependency order. Cleanup failure
is retained in the enclosing scope's error chain.

Dropping an uncompleted request, stream, or linear generator continuation
requests cancellation unless its protocol declares fire-and-forget delivery.
A protocol states whether cancellation is guaranteed, best effort, or
unsupported; a caller cannot assume stronger behavior.

## Waiting for alternatives

A scope may wait for the first acceptable result among several interactions.
The operation consumes a finite labeled product of pending interactions and
returns a union identifying the selected alternative. When several alternatives
are already available at the same logical point, declaration order is the
deterministic tie breaker.

Non-selected interactions remain owned by the scope. The caller retains or
explicitly cancels them; selection never silently detaches an interaction.
Timeouts are alternatives supplied by a clock or timer protocol, so they add
that protocol's effects and dependencies rather than reading ambient time.

## Backpressure and queue bounds

Every event or request protocol declares its admission behavior: bounded wait,
bounded rejection with `Result`, or contained loss for an isolated diagnostic
event. An ordinary `Unit` event may not silently discard a message unless its
protocol explicitly has isolated diagnostic semantics.

Queue capacity is an implementation choice only within the declared admission
behavior. A sender which may suspend or fail because of backpressure exposes
that interaction in its protocol and effect contract, allowing queue
dependencies to participate in deadlock checking.
