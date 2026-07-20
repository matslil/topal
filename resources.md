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

  destroy is fn ( file : File ) -> Result Unit
    operating-system close file.descriptor
```

The destructor belongs to `File`, not to a particular constructor. Every
algorithm that constructs a `File` therefore produces a value with the same
lifetime behavior:

```topal
open-file is fn ( path : Path ) -> Result File
  operating-system open path
```

`destroy` is a declaration recognized by the language rather than an ordinary
algorithm that programs may call. The compiler invokes it when the final
reference to a value disappears. After the declared cleanup has run, the
compiler still destroys owned components and releases storage; user cleanup
does not replace those operations.

## Destructor results

A destructor has exactly one of these result types:

```topal
destroy is fn ( value : T ) -> Unit
destroy is fn ( value : T ) -> Result Unit
```

It cannot produce a replacement value. Operations such as `flush`, `commit`,
and `finish` are ordinary algorithms which may inspect or change the resource's
state without destroying its handle.

An infallible destructor makes leaving the final reference's scope infallible.
A fallible destructor makes that scope potentially fallible. Failure reports
that resource cleanup failed; it does not prevent Topal from destroying the
remaining components or releasing their memory.

When a body operation and one or more destructors fail, Topal preserves the
body failure as the primary error and records destruction failures in its
[contextual error chain](errors.md#causes-and-contextual-frames). Destruction
continues in deterministic reverse construction order so that one failure
cannot prevent the remaining cleanup.

## Passing and retaining resources

An algorithm which receives a value cannot assume that another reference will
remain after its call. Its local reference may be the final reference and may
therefore invoke the destructor when the algorithm returns. Accepting a type
with a fallible destructor consequently requires a fallible result, even when
the algorithm only observes the value:

```topal
metadata is fn ( file : File ) -> Result FileMetadata
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

Programs neither request these choices nor overload algorithms on them. In
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
the runtime work. The source syntax and algorithm contract do not depend on
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
