# Generators

Generators are resumable functions. They can yield values to a caller, receive
values when execution resumes, and eventually return a final value. This makes
the direction of every value explicit without associating generators with a
particular scheduler, message system, or collection representation.

## Declarations

A named generator uses `generator` rather than `fn` so that its calling
convention is visible at its declaration:

```topal
conversation is generator ( initial : Request )
  yields OutgoingRequest
  resumes IncomingResponse
  -> Result Conversation

  first-response : IncomingResponse is yield make-first-request initial
  second-response : IncomingResponse is yield make-second-request first-response
  finish-conversation first-response second-response
```

The three result directions are distinct:

- `yields` is the type sent from the generator to its caller;
- `resumes` is the type sent from the caller into the generator; and
- `->` is the type returned when the generator finishes.

Generator status is not inferred from the presence of `yield`. A `yield` in an
ordinary `fn` is an error for which the compiler may suggest changing the
declaration to `generator`.

## Starting and resuming

Applying a generator supplies its initial input and starts its execution:

```topal
conversation-value is conversation initial-request
```

There is no separate `start` operation. Execution proceeds until the generator
yields or returns. A yielded computation retains its own local continuation
state. Resuming it supplies the value produced by the suspended `yield`
expression:

```topal
response : IncomingResponse is yield outgoing-request
```

At this point `outgoing-request` is available to the caller. When the caller
resumes the computation with an `IncomingResponse`, the `yield` expression
evaluates successfully to that response and binds it to `response`.

Every `yield` has the effective type `Result Resume`, where `Resume` is the
declared `resumes` type. Its language-defined generator error vocabulary
contains `generator-closed`. A consumer which abandons the continuation resumes
the suspended `yield` with that error:

```topal
resume-result is yield outgoing-request

resume-result attempt
  Error ( code is generator-error generator-closed ) then
    finish-logical-operation ()
    return error generator-closed

  response : IncomingResponse then
    continue-with response
```

The generator may use this branch to finish or roll back logical work which is
not part of automatic resource cleanup. Success projection may instead
propagate `generator-closed` to the generator boundary. At that boundary the
runtime consumes it as the expected close signal because the consumer no
longer exists; it is not delivered as the generator's final application error.
Any other failure produced while handling closure follows the generator's
ordinary final result and structured-scope rules.

After a path observes `generator-closed`, it may return or propagate the close
signal but may not yield again. This is checked flow-sensitively. Once the
generator exits, destructors and other automatic cleanup run in dependency
order.

Conceptually, observing one generator step produces one of:

```text
Yielded ( yielded-value, continuation )
Returned return-value
```

The continuation accepts either a value of the declared `resumes` type or the
intrinsic `generator-closed` signal. It is linear: resuming the same
continuation more than once would duplicate a single logical computation and
is invalid. Only abandonment supplies `generator-closed`; ordinary source code
does not gain an explicit close or cancellation operation. The compiler may
implement continuations as mutable state machines when doing so does not change
their observable behavior.

A generator is permitted to return before its first `yield`. Code driving a
generator must therefore account for both `Yielded` and `Returned`; generator
application does not promise that a value will be yielded.

## Unit-resumed traversal

A generator with `Unit` as its resume type needs no value from its consumer
between yielded values. Such a generator supports `foreach` directly:

```topal
generator-value is values first

result is generator-value foreach { value }
  print value
```

`foreach` applies its inferred anonymous function once for each yielded value,
resumes the generator with `Unit`, and stops when the generator returns. The
`foreach` expression produces the generator's final return value, which may be
ignored when it is `Unit`.

Conceptually, its classification is:

```text
foreach : Generator Yield Unit Return, ( Yield -> Unit ) -> Return
```

If evaluating the body can fail, traversal stops without resuming the
continuation again and the enclosing result accounts for that failure. More
general loop forms and bidirectional generator-driving conveniences remain
open; `foreach` specifies only the common `Unit`-resumed case.

Leaving `foreach` before the source returns abandons its continuation. The
suspended `yield` receives `generator-closed`, the generator performs any
explicit close handling, and automatic cleanup follows. The enclosing scope
waits for the close path and retains failures from its shutdown work or
cleanup.

## Generated traversal

`iterate` constructs a `Unit`-resumed generator from an initial value and an
function producing the next value. It yields the initial value first and is
unbounded by itself:

```topal
numbers is 0 iterate { value }
  value + 1
```

Consumers bound or terminate that computation. For example, `take-while`
yields the source prefix ending before its first rejected value:

```topal
0 iterate { value } value + 1
  take-while { value } value < 10
  foreach { value } print value
```

This is the compositional equivalent of a conventional loop with an initial
state, continuation condition, state update, and body. Unary `collect` may
materialize a finite generated traversal as a list when that value is actually
needed:

```topal
digits is collect
  0 iterate { value } value + 1
    take-while { value } value < 10
```

The more general `unfold` accepts a seed and a function returning either no
next step or a yielded value paired with the next seed:

```text
unfold : S, ( S -> Option ( T, S ) ) -> Generator T Unit Unit
```

Unlike `iterate`, `unfold` can finish from its own step result and may keep its
internal state distinct from its yielded type. Neither operation constructs an
intermediate collection.

## Independence from message passing

Generators provide control flow and do not assign meaning to yielded values.
A consumer may calculate a resume value immediately, retain a continuation for
later, or select among several continuations. Message-passing infrastructure
can consequently use generators without generators depending on queues,
messages, tasks, or a scheduler.

When a task handler suspends, a task runtime may retain the handler
continuation and resume it when the corresponding event arrives. The generator
need not receive or return a scheduler context. Its owning task identity remains
stable across resumptions, while typed endpoints and protocols provide the
authority to communicate with other tasks.

Task identity does not itself grant unrestricted access to task state or to
another task's queue. The language and runtime remain responsible for making
task state available only to the currently executing handler segment and for
restoring current state when a continuation resumes. A continuation may retain
ordinary local values, endpoint capabilities, and protocol identifiers, but
not hidden access to another task's mutable implementation state.
