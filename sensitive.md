# Sensitive values

Topal lets a programmer mark a value as sensitive when it contains a password,
private key, access token, or other information which should not leave the
application accidentally. Sensitivity is a compiler-checked value qualifier,
not a different semantic type and not an encryption mechanism.

Provisional syntax places `sensitive` on the binding:

```topal
sensitive password is read-password
sensitive private-key : PrivateKey
private-key is load-private-key configuration
```

The qualifier is visible in source and retained in compiled contracts, but it
does not change the value's ordinary type. A sensitive `String` remains a
`String`, and sensitivity does not select an overload or change equality.

## Propagation

Sensitivity follows the marked information. Copying, moving, or borrowing a
sensitive value preserves the qualifier:

```topal
sensitive password is read-password
copy is password
authenticate-locally copy
```

Both `password` and `copy` are sensitive. The same rule applies when a
sensitive value is stored in a field or collection, returned unchanged from an
algorithm, destructured, or selected through another view. A containing value
retains which of its reachable information is sensitive, so passing the whole
container to a boundary cannot hide a sensitive component.

The qualifier does not spread merely because a sensitive value was used during
a computation. A newly constructed result is not automatically sensitive:

```topal
sensitive password is read-password
accepted is verify-password password
```

`accepted` is not sensitive unless the programmer marks it. An algorithm which
copies, moves, borrows, or returns sensitive input without constructing new
information cannot remove the qualifier by changing bindings or hiding the
operation behind a call. If a newly constructed result is itself secret, its
binding or the algorithm's result contract must mark it explicitly.

There is no implicit declassification operation. Removing a `sensitive`
qualifier from information which still carries it is a compile error.

## Local use

An ordinary algorithm which acts only within the application may freely accept,
inspect, compare, store, and otherwise use sensitive values. It does not need a
sensitivity annotation:

```topal
password-matches is fn (
  supplied : String,
  expected : String
) -> Bool
  constant-time-equal ( supplied, expected )
```

Calling `password-matches` with sensitive inputs is valid. Its newly constructed
`Bool` result is not automatically sensitive. Local calls therefore do not
require sensitivity annotations to propagate mechanically through the call
tree.

## Application boundaries

The compiler distinguishes local computation from an operation which can send
information outside the application. Such boundaries include files, sockets,
child processes, foreign calls, exported application interactions, and
externally collected diagnostics. Passing sensitive information to a boundary
is rejected unless the corresponding boundary parameter explicitly declares
that it accepts sensitive arguments.

Provisional syntax places `sensitive` on each boundary parameter which accepts
sensitive information:

```topal
send-credential is fn (
  connection : Connection,
  sensitive credential : String
) -> Result Completed
  protocol-send ( connection, credential )
```

The qualifier states that the boundary implementation expects that parameter
to contain sensitive information and is responsible for the corresponding
handling policy. Other parameters remain unqualified and reject sensitive
arguments. The qualifier permits the transfer; it does not promise encryption,
redaction, access control, or secure erasure. Those guarantees require their
own types, protocols, and effects.

An unqualified boundary rejects a sensitive value:

```topal
sensitive password is read-password
print password # Compile error: print does not accept sensitive information.
```

The check applies to direct arguments and to sensitive information reachable
through records, collections, errors, closures, and other containers. Copying,
moving, borrowing, or placing a value in another container cannot bypass the
boundary check. A transformation which constructs a new result does not
automatically propagate sensitivity; the programmer is responsible for marking
that result when it too contains secret information.

A sensitive parameter is part of the algorithm contract and is preserved
through aliases, higher-order parameters, protocols, and foreign declarations.
A local implementation does not mark a parameter merely because it receives a
sensitive value. The parameter qualifier is required when an
application-boundary function may release sensitive information received
through that parameter.

## Diagnostics and generated support

Compiler-generated diagnostics must not print the contents of sensitive values.
They may identify the binding, type, source location, and rejected boundary.
Tracing, logging, panic presentation, core dumps, and similar generated support
must omit or redact sensitive contents. They may expose those contents only
through an external operation whose corresponding parameter explicitly accepts
sensitive information.

Sensitivity is intended to prevent accidental disclosure. It does not defend
against a deliberately malicious algorithm which derives new unmarked
information from a secret, nor does it replace platform memory protection,
secret storage, cryptography, or review of explicitly sensitive boundaries.
