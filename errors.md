# Results and errors

Topal reports failure explicitly with `Result`; exceptions are not part of the
language. Errors have a common structured representation, while each fallible
function declares the error-code vocabulary it may return.

## Result

`Result Value` represents either a successful `Value` or an `Error`:

```text
Result Value = Value + Error
```

Fallibility is part of a function's explicit interface. Input and output types
are mandatory and are not inferred from its body:

```topal
increment is fn ( value : Integer ) -> Integer
  value + 1

read-count is fn ( path : Path ) -> Result Integer
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

The compiler tracks a function's permitted code subtype separately as part of
its `Result` contract. It does not parameterize `Error`, derive an
`Error FileErrorCode` type, or require callers to inspect the type of an error.

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
functions. A fallible function must have an `ErrorCode` type available when its
`Result` contract is declared. The compiler records the exact resolved type as
part of that function's interface.

An implementation boundary may contribute an additional language-defined code
to that declared vocabulary. In particular, a task request or stream adds
`task-terminated`, from Topal's stable task error domain. The operation still
has one `Result Value`; its effective code set is the union of the application
vocabulary and this task code.

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

Different functions in the same source file may use different `ErrorCode`
types. The connection is between a function and the type resolved for its
contract; it does not move the type into the function's namespace or create a
new subtype there.

A scope may bind an imported definition to the required local name:

```topal
use shared-error
ErrorCode is shared-error ErrorCode
```

This is an alias, not a copy or conversion. Both names denote the same enum and
its values retain the same identity.

The code vocabulary is part of a fallible function's public type. When the
function is exported, its `ErrorCode` definition must also be exported through
some reachable namespace. It need not be exported under the function's
namespace, but a consumer must be able to resolve the exact type in order to
use the function. The compiler rejects a public fallible function whose
`ErrorCode` remains private.

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

## Success projection and propagation

When an expression has type `Result Value` but its immediate context explicitly
requires `Value`, Topal projects the successful value and returns an error from
the current function unchanged:

```topal
load-configuration is fn ( path : Path ) -> Result Configuration
  text : String is read-file path
  parse-configuration text
```

If `read-file path` succeeds, its `String` is bound to `text`. If it fails, the
error is returned immediately from `load-configuration`. The enclosing
function must declare a compatible `Result` and `ErrorCode` contract. It may
use the same shared code type or explicitly translate or wrap the error into
its own code vocabulary.

Merely binding, passing, or returning a `Result` does not project it:

```topal
attempt is read-file path
```

Here `attempt` retains type `Result String`. Projection is requested only by a
context that explicitly requires the success type. An infallible function
cannot project a result because it has nowhere to return the error:

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

An error domain identifies the stable vocabulary which owns an `ErrorCode`.
Domains are generated from qualified module or namespace scope by default. A
module may explicitly use another domain when several modules intentionally
share a vocabulary.

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
which semantic vocabulary owns it. Source locations do not participate in
error equality or the stable API contract.

A standard formatter presents the outer operation first and then follows its
causes. It combines the printable domain, converted code description, and
optional detail for each frame. Diagnostic presentation must account for
sensitive details and may redact them without changing error identity.
