# Results and errors

This document records the provisional error model for Topal. Algorithms report
failure explicitly with `Result`; exceptions are not part of the language.
The model standardizes the shape of errors so that results from different
modules compose without erasing the context needed to diagnose a failure.

## Result

`Result` represents either a successful value or an `Error`. Its error shape is
standard rather than chosen independently by every algorithm. This lets generic
algorithms forward failures and lets callers inspect errors consistently across
module boundaries.

Fallibility is part of an algorithm's explicit interface. Input and output types
are mandatory in an algorithm declaration and are not inferred from its body:

```topal
increment is fn ( value : Integer ) -> Integer
  value + 1

read-count is fn ( path : Path ) -> Result Integer
  body
```

`increment` cannot return or propagate an error. `read-count` may return either
an `Integer` or the standard `Error`. `Result` is never added implicitly to a
declared output type.

The exact type and construction syntax remains provisional. Conceptually:

```topal
Result Value

Error
  domain : ErrorDomain
  code : ErrorCode
  detail : Optional String
  cause : Optional Error
  source : Optional SourceLocation
```

`detail`, `cause`, and `source` are absent when they do not add information.

## Success projection and propagation

When an expression has type `Result Value` but its immediate context explicitly
requires `Value`, Topal projects the successful value and returns an error from
the current algorithm unchanged. The enclosing algorithm must explicitly
declare a compatible `Result` output:

```topal
load-configuration is fn ( path : Path ) -> Result Configuration
  text : String is read-file path
  parse-configuration text
```

If `read-file path` succeeds, its `String` is bound to `text`. If it fails, the
error is returned immediately from `load-configuration`. This is success
projection with error propagation, not a general implicit conversion from
`Result String` to `String`.

Merely binding, passing, or returning a `Result` does not project it:

```topal
attempt is read-file path
```

Here `attempt` retains type `Result String`. Projection is requested only by a
context that explicitly requires the success type, such as the `String`
classification above. An algorithm with an infallible output cannot use this
projection because it has nowhere to return the error:

```topal
load-length is fn ( path : Path ) -> Integer
  text : String is read-file path  # error: load-length is infallible
  text length
```

Explicit matching remains available when the caller needs to recover from,
translate, or otherwise inspect an error rather than propagate it.

Propagation normally preserves the error unchanged. Intermediate algorithms do
not add frames merely because they project a success value. When an error leaves
a public top-level operation, that operation still adds the mandatory contextual
frame described below.

## Domains and codes

An error domain identifies a stable vocabulary of errors. Domains and codes are
objects with stable identity, not arbitrary strings, so programs can compare
them reliably. Both provide printable descriptions for diagnostics.

Domains are generated from qualified module or namespace scope by default:

```topal
example.configuration
example.network.http
```

A domain is not generated for every file or algorithm. File paths and algorithm
names describe implementation locations and change during refactoring; they do
not define independently useful error vocabularies. A module may explicitly use
another domain when several modules intentionally share a vocabulary.

A code identifies a semantic category within its domain:

```topal
example.configuration.could-not-apply
example.configuration.invalid-value
```

Normal control flow matches the domain and code of the outermost error. Codes
from causes remain available for deliberate inspection and diagnostics, but do
not form a composite code.

## Details and descriptions

The domain and code supply a default human-readable description. An optional
detail distinguishes a particular occurrence without requiring a new code:

```text
configuration: could not apply: configuration "production"
```

Details supplement the code description rather than repeat it. They may include
information such as a configuration name, input position, or rejected value.
Programs must not parse descriptions or details to determine behavior.

Details are intended for people and may be localized. Values which need
independent inspection, filtering, localization, or redaction may eventually
require structured diagnostic context; the initial error model does not make a
free-form string part of the programmatic error identity.

## Causes and contextual frames

An algorithm may wrap an error by returning a new error whose `cause` is the
original. Each frame retains its own domain, code, and optional detail. Walking
the causes produces a semantic trace of the operations that failed:

```text
configuration: could not apply: configuration "production"
caused by configuration reader: could not read: "production.conf"
caused by file: not found: "production.conf"
```

This trace records abstraction boundaries and attempted operations rather than
every function call. An intermediate layer that merely forwards a failure does
not add a frame. It adds one only when it contributes meaningful operational or
semantic context.

The public, top-level operation is the exception to that forwarding rule. It
always adds a frame when returning a failure, even if it can only describe the
operation that was attempted. Without that frame, a receiver might know that a
file was missing but not whether the application was loading configuration,
opening a document, or applying an update.

## Abstraction boundaries

The outermost domain and code are the error contract of the current API. A
public layer wraps a lower-level failure when its own operation fails, rather
than changing the identity of the underlying error. The original error remains
as the cause.

Consequently, a private implementation domain can remain in the diagnostic
chain without requiring callers to depend on it. Callers normally handle the
public frame:

```text
example.configuration: could not apply
caused by example.internal.parser: invalid value
```

Replacing the private parser can change the inner diagnostic frame without
changing the public error contract. APIs that intentionally expose a lower-level
abstraction may instead forward its error unchanged below their mandatory
top-level operation frame.

## Source locations

Source locations answer where an error was constructed, while domains answer
which semantic vocabulary owns it. The compiler may attach a file, algorithm,
and line as diagnostic metadata, potentially only in diagnostic builds. Source
locations do not participate in error equality or the stable API contract.

## Presentation

A standard formatter presents the outer operation first and then follows its
causes. It combines the printable domain, code description, and optional detail
for each frame. Applications may choose a concise presentation for end users
and retain the full chain and source locations for logs or diagnostics.

Diagnostic presentation must account for sensitive details. Retaining a cause
does not require exposing every private frame or value to every user; an
application or boundary may redact presentation while preserving the error for
authorized diagnostics.
