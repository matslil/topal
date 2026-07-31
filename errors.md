# Results and errors

Topal reports failure explicitly with `Result`; exceptions are not part of the
language. Errors have a common structured representation, while each fallible
function declares the error-code vocabulary it may return.

## Result

`Result` takes the success type and one explicit error-vocabulary component:

```text
Result ( Value, Errors ) = Value + Error
```

`Errors` is either one concrete `ErrorCode` subtype, a product of several such
types, or the empty product `()`. For example:

```topal
Result ( Document, FileErrorCode )

Result (
  Document,
  ( FileErrorCode, ParserErrorCode )
)
```

The compiler flattens nested error-vocabulary products, removes duplicate
nominal types, and treats their order as irrelevant. The first component is
always the successful value type; every member of the second component must be
an `ErrorCode` type. `Result Value` is invalid and no error vocabulary is
resolved implicitly from the surrounding scope.

The empty vocabulary is valid:

```topal
Result ( Completed, () )
```

It promises that the source operation declares no application error. The
wrapper may still be required by an implementation boundary, such as task
messaging, which contributes a documented language-defined vocabulary to the
effective compiled contract.

Fallibility is part of a function's explicit interface. Input and output types
are mandatory and are not inferred from its body:

```topal
increment is fn ( value : Integer ) -> Integer
  value + 1

read-count is fn ( path : Path ) -> Result ( Integer, FileErrorCode )
  body
```

`increment` cannot return or propagate an error. `read-count` may return either
an `Integer` or an `Error`. `Result` is never added implicitly to a declared
output type.

A task message implementation is permitted only where the operation already
declares `Result`. Crossing the task boundary does not implicitly wrap a plain
result; it extends the existing result's effective error-code vocabulary as
described below.

## Error representation

Every `Result` uses the common structured `Error` value. Conceptually:

```topal
Error
  domain : ErrorDomain
  code : ErrorCode
  detail : Optional String
  cause : Optional Error
  source : Optional SourceLocation
```

`detail`, `cause`, and `source` are absent when they do not add information.
`ErrorCode` is the common classification of error codes. Namespace-defined
enums are its subtypes, and the precise subtype varies between fallible
functions; the rest of the error representation remains common. The structured
representation can therefore retain causes carrying codes from other
vocabularies.

Putting a value of a concrete `ErrorCode` subtype in the `code` field does not
make the containing `Error` a subtype. Every error retains the same `Error`
type and representation. For example, a file code remains an `ErrorCode` when
stored in `Error.code`, and the containing value remains simply `Error`.

The compiler obtains a function's permitted code subtypes from the explicit
second component of its `Result` contract. It does not parameterize `Error`,
derive an `Error FileErrorCode` type, or require callers to inspect the type of
an error.

## Error-code vocabularies

Each concrete `ErrorCode` subtype is a finite
[enum](types.md#products-sums-and-enums) of identifiers. For example,
a file error vocabulary might contain:

```topal
not-found
permission-denied
invalid--path
```

The definition may live in any namespace and may be shared by unrelated
functions. A fallible function names every permitted vocabulary explicitly in
the second component of its `Result`. No specially named `ErrorCode` binding is
searched for in the function's scope.

Visible generic code may capture that complete second component symbolically:

```text
retry :
  ( () -> Result ( T, Errors ) )
  -> Result ( T, Errors )
```

If the generic body introduces another vocabulary, its result combines the
components:

```topal
Result ( T, ( Errors, RetryErrorCode ) )
```

Typed generic intermediate code retains `Errors` until final application
specialization. Explicit vocabulary parameters or bounds are needed only for
opaque implementations, foreign boundaries, abstraction requirements, or
otherwise uninferable relationships.

An implementation boundary may contribute an additional language-defined code
to that declared vocabulary. In particular, a task request or stream adds
`task-terminated`, from Topal's stable task error domain. An operation declared
as `Result ( Value, ApplicationErrors )` therefore has the effective task
contract:

```topal
Result (
  Value,
  ( ApplicationErrors, TaskErrorCode )
)
```

This widening is implementation evidence rather than a mutation of the source
interface. A direct implementation exposes only the codes it declares. A task
implementation exposes the union, and a dynamic choice among implementations
exposes the union needed for every remaining alternative. The compiler uses
that effective closed set for matching and propagation checks.

Task messaging adds no general unavailable, admission, or transport error.
Every endpoint denotes an application-local task instance which existed.
External communication belongs to a local mirror task, whose interface
declares its application-level failures. If the mirror itself terminates before
committing a reply or final stream result, the caller receives
`task-terminated`; its structured reason or cause may retain the underlying
failure.

The initiating `Result ( Completed, TerminationErrorCode )` session of
`terminate-cleanly` is the one termination exception. The runtime retains it
while other queued and new requests receive `task-terminated`, then returns
`Completed` only after the lifecycle handler and cleanup finish. A termination
or cleanup failure uses that retained result instead.

A `yield` expression similarly adds `generator-closed`, from Topal's stable
generator error domain, to its declared resume value. This code is supplied
only when the consumer abandons the linear continuation. Generator code may
match it to perform deliberate shutdown work. If it reaches the generator
boundary, the runtime consumes it as closure rather than exposing it as a final
application error to a consumer which no longer exists. Other shutdown and
cleanup errors retain their ordinary scope behavior.

Accessing a `Weak T` adds `weak-unavailable`, from Topal's stable weak-reference
error domain, to the resulting `Result ( T, WeakErrorCode )`. The access
atomically either retains an ordinary `T` or reports that the target is no
longer retainable. A successful retained value remains valid for its lexical
lifetime; `weak-unavailable` cannot subsequently replace it.

Applying `with-timeout` to a reply-waiting expression adds `timeout-occurred`
from `TimeoutErrorCode`. It merges the vocabulary into an existing message
result rather than adding another wrapper:

```topal
5[s] with-timeout ( network request packet )

Result (
  Response,
  (
    NetworkErrorCode,
    TaskErrorCode,
    TimeoutErrorCode
  )
)
```

For a message stream, the vocabulary is added only to the stream's final
`Result`. Individual yields keep their declared type. If waiting for a yield or
final return times out, the stream ends with this timeout error; values already
yielded remain observed.

When the immediate wait returns a non-`Result` value `T`, such as the labeled
product from `match-all`, the timeout construction returns
`Result ( T, TimeoutErrorCode )`. A group timeout around `match-first` or `match-all`
returns no partial response value.

The timeout introduced by this caller uses the stable `lang with-timeout`
domain. A handler may return the same `TimeoutErrorCode` value under its own
domain. Duplicate code vocabularies collapse in the `Result` component while
the runtime domains remain distinct.

A timeout error reports only that the caller's observation deadline expired.
It does not report whether the underlying operation committed or which of its
declared effects occurred. That uncertainty remains in effect and protocol
evidence rather than creating another portable timeout error code.

Different functions in the same source file may name different error
vocabularies directly. Naming a type in `Result` does not move it into the
function's namespace or create a new subtype there. Ordinary aliases remain
available for long or repeated vocabulary products:

```topal
DocumentErrors is ( FileErrorCode, ParserErrorCode )

load-document is fn (
  path : Path
) -> Result ( Document, DocumentErrors )
  body
```

The complete vocabulary product is part of a fallible function's public type.
When the function is exported, every vocabulary it names must also be exported
through a reachable namespace. The compiler rejects a public fallible function
whose result names a private vocabulary.

## Error-code descriptions

The compiler knows that every `ErrorCode` value can be converted to `String`.
Conversion starts with the source identifier and scans hyphens from left to
right:

- a single `-` becomes a space;
- a pair `--` becomes one literal `-`.

For example:

```text
not-found         -> "not found"
permission-denied -> "permission denied"
invalid--path     -> "invalid-path"
```

The identifier is programmatic identity; its converted string is the default
human-readable description. Programs match the identifier rather than parsing
the string.

## Matching error codes

Patterns inspect the `code` field using ordinary nominal enum matching.
Different lines may name values belonging to different `ErrorCode` subtypes:

```topal
attempt
  Error ( code is file-error not-found ) then recover ()
  Error ( code is parser-error invalid-syntax ) then reject ()
  Error problem then report problem
```

The qualified code value identifies its namespace-defined enum. This is a
value-pattern test on `Error.code`, not type introspection on `Error`. The
compiler checks each pattern against the code vocabularies in the function's
statically recorded result contract. When that contract describes a closed
set, it can also check the match for exhaustiveness; an unconstrained dynamic
`Error` requires a fallback branch.

Code identifiers are never global. A timeout pattern, for example, names the
scope which publishes `TimeoutErrorCode`:

```topal
attempt
  Error (
    domain is lang with-timeout,
    code is timeout-error timeout-occurred
  ) then handle-caller-timeout ()

  Error (
    domain is network request,
    code is timeout-error timeout-occurred
  ) then handle-handler-timeout ()
```

Matching only the qualified code accepts that condition from every domain.
Matching the domain distinguishes which operation or abstraction reported it.
Two identically spelled values from different `ErrorCode` scopes remain
different enum values.

## Success projection and propagation

When an expression has type `Result ( Value, SourceErrors )` but its immediate
context explicitly requires `Value`, Topal projects the successful value and
returns an error from the current function unchanged:

```topal
load-configuration is fn (
  path : Path
) -> Result (
  Configuration,
  ( FileErrorCode, ParserErrorCode )
)
  text : String is read-file path
  parse-configuration text
```

If `read-file path` succeeds, its `String` is bound to `text`. If it fails, the
error is returned immediately from `load-configuration`. The enclosing
function must declare a `Result` whose explicit vocabulary contains every
propagated source vocabulary. It may instead translate or wrap the error into
one of its own declared vocabularies.

Merely binding, passing, or returning a `Result` does not project it:

```topal
attempt is read-file path
```

Here `attempt` retains the complete result type of `read-file`, including its
explicit error-vocabulary component. Projection is requested only by a context
that explicitly requires the success type. An infallible function cannot
project a result because it has nowhere to return the error:

```topal
load-length is fn ( path : Path ) -> Integer
  text : String is read-file path  # error: load-length is infallible
  text length
```

Explicit matching remains available when the caller needs to recover from,
translate, or inspect an error rather than propagate it.

Propagation normally preserves the complete error unchanged. Wrapping creates
a new outer error and retains the original as its cause.

## Domains, details, and causes

An error domain identifies the stable operation, subsystem, or abstraction
which reported an error. An `ErrorCode` independently identifies what occurred.
The same code vocabulary may be used by several domains, and one domain may
report codes from several vocabularies declared by its public `Result`
contracts. Domains are generated from qualified declaration scope by default;
an API may explicitly select a stable domain shared by several declarations.

An optional detail distinguishes a particular occurrence without requiring a
new code. Details supplement the code description rather than repeat it.
Programs must not parse descriptions or details to determine behavior.

A function may wrap an error by returning a new error whose `cause` is the
original. Each frame retains its own domain, code, and optional detail. Walking
the causes produces a semantic trace across abstraction boundaries:

```text
configuration: could not apply: configuration "production"
caused by configuration reader: could not read: "production.conf"
caused by file: not found: "production.conf"
```

An intermediate function which merely forwards a failure does not add a frame.
It adds one only when it contributes meaningful operational or semantic
context.

## Source locations and presentation

Source locations answer where an error was constructed, while domains answer
which operation, subsystem, or abstraction reported it. Source locations do not
participate in error equality or the stable API contract.

A standard formatter presents the outer operation first and then follows its
causes. It combines the printable domain, converted code description, and
optional detail for each frame. Diagnostic presentation must account for
sensitive details and may redact them without changing error identity.
