# Resource lifetime and destruction

Topal applies the lifetime safety used for allocated memory to every value that
refers to an external resource. A file, socket, transaction, process, or device
is still an ordinary value of an ordinary type. The source language does not
distinguish unique, borrowed, and reference-counted handles; the compiler
chooses transfers, borrowing, sharing, and their implementation while
preserving the lifetime behavior described here.

## Destructors belong to types

Every type has a destructor. The default destructor destroys the value's owned
components and releases its allocated storage. A type may define additional
cleanup for a resource it represents. Provisional syntax is:

```topal
File is type
  descriptor : FileDescriptor

  destroy is fn ( file : File ) -> Result ( Unit, ResourceErrorCode )
    operating-system close file descriptor
```

The destructor belongs to `File`, not to a particular constructor. Every
function that constructs a `File` therefore produces a value with the same
lifetime behavior:

```topal
open-file is fn ( path : Path ) -> Result ( File, ResourceErrorCode )
  operating-system open path
```

`destroy` is a declaration recognized by the language rather than an ordinary
function that programs may call. The compiler invokes it when the final
reference to a value disappears. After the declared cleanup has run, the
compiler still destroys owned components and releases storage; user cleanup
does not replace those operations.

## Destructor results

A destructor has exactly one of these result types:

```topal
destroy is fn ( value : T ) -> Unit
destroy is fn ( value : T ) -> Result ( Unit, ResourceErrorCode )
```

It cannot produce a replacement value. Operations such as `flush`, `commit`,
and `finish` are ordinary functions which may inspect or change the resource's
state without destroying its handle.

An infallible destructor makes leaving the final reference's scope infallible.
A fallible destructor makes that scope potentially fallible. Failure reports
that resource cleanup failed; it does not prevent Topal from destroying the
remaining components or releasing their memory.

When a body operation and one or more destructors fail, Topal preserves the
body failure as the primary error and records destruction failures in its
[contextual error chain](errors.md#domains-details-and-causes). Destruction
continues in deterministic reverse construction order so that one failure
cannot prevent the remaining cleanup.

## Passing and retaining resources

A function which receives a value cannot assume that another reference will
remain after its call. Its local reference may be the final reference and may
therefore invoke the destructor when the function returns. Accepting a type
with a fallible destructor consequently requires a fallible result, even when
the function only observes the value:

```topal
metadata is fn ( file : File ) -> Result ( FileMetadata, ResourceErrorCode )
  inspect file
```

This would not be a valid declaration when `File` has a fallible destructor:

```topal
metadata is fn ( file : File ) -> FileMetadata
  inspect file
```

The same rule applies indirectly. A composite type has a fallible default
destructor when destruction of any owned component can fail. A scope holding a
composite value must account for that potential failure.

Returning a resource, retaining it in another value, or otherwise extending its
lifetime can prevent destruction at a particular scope boundary. This may let
the compiler remove a destruction path, but source code is checked without
assuming that such a reference exists. An explicitly infallible result remains
a promise that neither the body nor destruction at its boundary can fail.

## Implicit ownership implementation

Ownership representation is not source-level API. Subject to the observable
lifetime rules, the compiler may:

- transfer a value whose old binding is no longer used;
- borrow a value for a call from which it cannot escape;
- introduce sharing when several uses remain;
- implement sharing with reference counts or another safe representation; and
- remove retains, releases, and destructor checks it proves unnecessary.

Programs neither request these choices nor overload functions on them. In
particular, a public type is not exposed as a `Shared File` or an `Owned File`.
The compiler must conservatively retain the possibility that releasing any
reference is the release of the final one. Optimization may prove otherwise,
but that proof does not weaken the source-level function contract.

Because a fallible destructor is observable, the compiler must preserve which
scope receives its error and its ordering relative to other effects. Changing
between transfer and sharing must not move a possible destruction failure
across a function boundary.

## State checks

Resource state is handled like other object state. An operation may check at
runtime that a file is writable or that a transaction is active. When the
compiler can prove the same fact, it may perform the check statically and omit
the runtime work. The source syntax and function contract do not depend on
which kind of proof is available.

This separates state transitions from lifetime termination. Committing a
transaction can leave a useful value recording the completed transaction, and
flushing a file does not remove references to the file. Only disappearance of
the final reference invokes its destructor.

Failures important to a program should normally be exposed by ordinary
operations at the point where the program can respond to them. A fallible
destructor remains available for resources, such as files, whose underlying
system can report a new failure only during final cleanup. The enclosing scope
then handles or forwards that failure through its declared `Result`.

## Lexical resource lifetime

Acquisition is an ordinary fallible operation on the resource context. Its
success continuation introduces the resource binding and therefore its lexical
lifetime:

```topal
result is file-system open-file path { file }
  process file
```

Here `file-system` selects the filesystem resource, `open-file path` returns a
`Result` carrying `file` on success, and `{ file }` binds that successful value
as the argument of the indented continuation. The body does not run when
acquisition fails. When the body does not return the resource, its retained
reference reaches the ordinary lexical boundary and may invoke the resource
destructor.

The acquisition success is consumed by the continuation binding rather than
becoming a second outer success value. Consequently, acquisition and cleanup
contribute completion plus their error vocabularies, while the body's success
value becomes the enclosing result. This is the anonymous
[`Result` composition](errors.md#composing-results) rule rather than special
resource syntax.

Returning the resource, or a value which contains it, is an ordinary explicit
escape:

```topal
connection is network connect address { connection }
  authenticate connection
  connection
```

The compiler applies its normal ownership analysis. When the old binding is no
longer used, this is a move into the receiving scope; no separate transfer
keyword, capability, or public ownership type is involved. The same rule
applies when the resource is nested in a tuple, record, variant, or another
resource-owning value. If other references remain, the compiler may use safe
sharing while preserving the same observable lifetime.

An escaped resource is not destroyed at the inner scope boundary.
Responsibility for its eventual destruction, including any destructor failure,
follows it to the scope containing its final reference. Acquisition and body
failures compose into the enclosing result; destruction failures from resources
which do not escape are reported there as well. A borrow or hidden alias cannot
make a resource escape because borrowed values are not permitted to outlive
their source.

When destruction occurs at the continuation's lexical boundary, the result
accounts for acquisition, body, and destruction failure. A body failure remains
primary and destruction failures become contextual causes. This gives close,
rollback, or final flush failure a predictable handling point without making an
unrelated observer the accidental final-reference boundary.

Topal defines no generic `with-resource` operation. Ordinary continuation
binding, ownership, lexical scope, and destruction already provide its proposed
behavior. Libraries may still define policy-specific higher-order functions
such as `with-transaction`, `with-connection`, or `with-temporary-directory`
when they add behavior such as commit and rollback, pooling, restoration, retry,
or error translation rather than merely repeating lexical cleanup.

## Resource cycles

Automatic sharing must not make destruction depend on collecting an
unobservable strong-reference cycle. A type whose destructor owns an external
resource cannot participate in a possible cycle of owning references unless
the compiler proves that the cycle is broken before its scope exits.

Long-lived back references use the language-defined `Weak` capability-backed
construction:

```topal
Window is type
  controls : List Control

Control is type
  window : Weak Window
```

Constructing `Weak Window` from an accessible `Window` records a non-owning
reference. It does not keep the window alive and does not participate in an
owning resource cycle. Destroying or copying the weak value has no effect on
the target's lifetime. The source model does not expose reference counts,
addresses, or garbage-collector timing.

Access to a `Weak T` atomically attempts to retain an ordinary `T` and has
effective type `Result ( T, WeakErrorCode )`. If the target is no longer
retainable, it returns the language-defined `weak-unavailable` code from Topal's
stable weak-reference error domain:

```topal
window : Window is control window
```

The successful `Window` remains alive for its ordinary lexical lifetime.
Returning or storing it may extend that lifetime according to the normal move
and safe-sharing rules. If doing so would establish a possible owning resource
cycle, the existing cycle check rejects it.

A block form retains the target once for a complete region:

```topal
control window { window }
  update-title window
  redraw window
```

When the weak value is already locally named, the same form is:

```topal
window { window }
  update-title window
  redraw window
```

The binding inside the block is an ordinary retained `Window`, not another
weak value. Failure to retain prevents the block from running and returns
`weak-unavailable`. On every successful path the reference remains valid for
the whole block, avoiding races between separate weak accesses. Leaving the
block releases it and accounts for a possible final-reference destructor
failure. Returning the retained value from the block instead moves its lifetime
into the receiving scope.

`Weak T` may also refer to an owning task instance when code needs optional
non-owning monitoring access. Successful promotion retains the ordinary task
value for the access region; a request may nevertheless return
`task-terminated` if task termination has already committed. The task's final
result remains owned by its original implicit join obligation.

`Weak TaskType` is distinct from a task `Endpoint`. Weak access begins with
`weak-unavailable` or an ordinary retained task value. An endpoint instead
retains a restricted messaging authority directly and reports
`task-terminated` when its task has ended; it is not itself promoted through
weak access.

Pure immutable values may use cyclic representations internally when their
observable semantics remain finite. Recursive source values are nevertheless
constructed algebraically rather than through mutable cyclic references.
