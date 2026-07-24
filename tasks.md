# Tasks and intrinsic messaging

Every runtime computation in Topal executes in a task. A task owns private
state and receives interactions through algorithms declared directly in its
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

Algorithms called by a handler normally execute in the same task. Only a call
through another task capability crosses a task boundary. Library algorithms
therefore remain ordinary reusable algorithms and do not need to be declared as
tasks.

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
algorithm. Topal creates the root task and invokes its `start` handler with the
application arguments:

```topal
# application.t
use lang topal-unix

configuration : Configuration
server : Server

start is fn (
  arguments : CommandArguments
) -> Result Completed

  configuration is load-configuration arguments
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
feature may provide command arguments, signals, and orderly shutdown, while an
Android feature may provide its platform lifecycle and application events.
The feature specifies recognized handler names, their types, ordering, and
delivery guarantees. Other algorithms in `application.t` remain private helper
algorithms executed by the root task.

Operating-system and framework adapters hold restricted capabilities for this
application protocol; they do not gain access to arbitrary application
algorithms or state. Events arriving during startup are queued until `start`
finishes, so platform handlers cannot observe partially initialized state.

## Runtime scope

All runtime Topal code has a current task identity and task scope. Ordinary
algorithm calls inherit the caller's task, child tasks belong to structured
task scopes, and foreign callbacks enter Topal by delivering a typed task
interaction. Static evaluation constructs compile-time objects and is not
runtime execution, so it does not require a runtime task.
