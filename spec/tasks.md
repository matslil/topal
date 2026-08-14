# Tasks and messaging

## Formal text

### TOPAL-TASK-DEFINITION-001 — Task definitions

Applying `Task` to an option record constructs a specialized task classifier.
An indented declaration classified by that value constructs a task definition.
The definition shall contain exactly the state fields and handlers declared in
its body and shall contain at least one `start` handler. State fields are
private and are selectable only through current-context `@` inside execution
belonging to the task.

### TOPAL-TASK-HANDLER-001 — Handler shapes

`start` is a lifecycle handler and shall not be exposed as a message operation.
Every ordinary message handler shall declare `MessageContext` as its leading
input and zero or one ordinary payload inputs after it. A function returning
`Unit` is an event. A function with a response channel shall return `Result`;
its success classifier shall be `Completed` or a non-`Unit` value.

### TOPAL-TASK-STATE-001 — Atomic state replacement

Within a task handler, `@ field is value` constructs a replacement for that
declared state field. The replacement shall satisfy the field classifier, and
the complete resulting task state shall satisfy every declared state-field
classifier before it becomes observable. An unqualified binding never replaces
task state.

### TOPAL-TASK-LIFECYCLE-001 — Construction and start

Applying a task definition evaluates its argument as the operand of `start`,
executes `start`, and produces a fresh owning task instance only after every
state field has been initialized with a conforming value. Each produced
instance has a distinct runtime identity even when its definition identity is
shared.

An optional `terminate` lifecycle handler is owner-only and executes at most
once. After termination commits, queued/new Unit events are discarded and
requests or streams which have not committed their final result produce
`task-terminated` in the `lang task` error domain.

### TOPAL-TASK-MESSAGE-001 — Message transaction

Applying `instance operation payload` shall select only a handler published by
the instance definition, create one stable transaction identity, supply a
compiler-created `MessageContext`, and deliver the payload to that handler.
State changes made by a successful handler become the instance's next state.
The formal trace shall associate send, receive, handler execution, and any
response with the same transaction identity so a debugger can follow the
transaction as one source-level call-like transition.

A generator handler establishes a stream transaction. Its leading
`MessageContext` and optional payload are delivered once, each yield is sent to
the caller, each resumption is delivered back to the serving task, and its
required `Result` final return commits the transaction. Suspending at a yield
releases state authority; resumption reacquires and observes current task state.

The reference interpreter may execute delivery immediately in its deterministic
scheduler. This does not add a completion dependency to a `Unit` event; the
observable event result remains `Unit` as required by
`TOPAL-CONC-INTERACT-001`.
