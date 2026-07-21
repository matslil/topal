# Modules, scopes, and visibility

Topal source layout defines a hierarchy of scopes. The source root is the
scope of an application or library, directories form nested scopes, and
ordinary source files form scopes within their containing directory. Functions
and other values are the terminal names in this hierarchy.

This model keeps the source tree and canonical names aligned while allowing a
module to expose an interface that does not mirror its implementation layout.

## Source layout and canonical names

Except for `module.t`, each source filename contributes one component to the
scope path. The `.t` suffix is not part of the component. Given:

```text
application/
├── main.t
└── logger/
    ├── module.t
    └── emit-logs.t
```

the ordinary files introduce the scopes `main` and `logger emit-logs`. A
public function named `err` in `emit-logs.t` consequently has the name:

```topal
logger emit-logs err
```

A scope name acts as a prefix for either another scope or a terminal value,
including an algorithm. Name resolution walks through scope components until
it reaches a value. Any remaining expressions are applied to that value:

```topal
logger emit-logs err Message
```

Here `logger` and `emit-logs` resolve scopes, `err` resolves an algorithm, and
`Message` is its input.

## Making scopes available

`use` makes a published path available to the current source file. When the
path ends at a scope, it has an implicit recursive wildcard, but it does not
flatten the hierarchy:

```topal
use logger

logger emit-logs err Message
logger configure destination
```

The single `use logger` is sufficient for all names published beneath that
scope. Keeping the `logger` prefix prevents collisions and makes the origin of
a name visible at its use site.

A narrower path can be used when a source file depends on only part of an
interface:

```topal
use logger emit-logs
```

The path may also end at a terminal value:

```topal
use logger set-logger
```

Making a scope available does not bypass visibility. The implicit wildcard
ranges over the module's published interface, not every file physically below
its directory.

## Scope bindings

`is` can bind a local name to a scope in the same way that it binds a name to
any other object:

```topal
use logger

logging is logger emit-logs
logging err Message
```

The resolver consumes `logging` as a scope prefix, resolves `err` within that
scope, and then applies the resulting algorithm to `Message`. Bindings can
shorten a path or give it a name appropriate to the current module:

```topal
diagnostics is logger emit-logs
diagnostics warning Message
```

Aliasing a scope neither copies its contents nor changes their visibility.

## Private by default

Every declaration in a source file is private by default. `pub` makes a name
visible across its nearest enclosing module boundary. For algorithms, `pub`
occupies the modifier position used by `static`:

```topal
set-logger is fn pub ( logger : Logger ) -> Logger
  implementation
```

For other classified values, it prefixes the declaration while the ordinary
classification continues to describe the binding's type:

```topal
pub destination : String
```

The precise ordering when modifiers are combined remains part of the
provisional surface syntax.

Publication can also name an existing binding. This supports both values and
scopes without a separate export or re-export construct:

```topal
logging is logger emit-logs
pub logging
```

The published name is `logging`; the original path remains an implementation
detail. An individual value can be renamed in the same way:

```topal
error is logger emit-logs err
pub error
```

`pub` never reveals private contents reached through a scope binding. It makes
the bound scope itself available under the published name, preserving that
scope's interface.

## Visibility propagates one boundary at a time

Publication reaches only the nearest parent. A declaration must be published
again at each boundary it crosses. For example, a public `err` declared in
`logger/emit-logs.t` is available to the `logger` module, but not automatically
to users of `logger`.

The `logger` module can choose whether and how to expose it:

```topal
# logger/module.t

error is emit-logs err
pub error
```

A user of `logger` then sees:

```topal
use logger

logger error Message
```

This explicit propagation prevents a deeply nested declaration from becoming
part of an application's or library's interface merely because the leaf file
marked it public.

## The directory module

`module.t` is the exception to the ordinary filename rule. It represents its
containing directory and does not add `module` to the scope path. Definitions
in `logger/module.t` therefore belong directly to `logger`:

```topal
# logger/module.t

set-logger is fn pub ( logger : Logger ) -> Logger
  implementation
```

The canonical published name is:

```topal
logger set-logger
```

It is never `logger module set-logger`. A consumer may make the whole published
scope available:

```topal
use logger
logger set-logger configured-logger
```

or state the narrower dependency:

```topal
use logger set-logger
logger set-logger configured-logger
```

Consequently, `module.t` is both the implementation of the directory scope and
the place where that directory assembles its public interface. It may declare
members directly, publish selected members from child files, rename them, or
publish selected child scopes. The physical contents of a directory do not by
themselves determine its external interface.

## Resolution and visibility rules

The module model follows these rules:

1. The application or library source root is the top-level scope.
2. A directory introduces a nested scope.
3. An ordinary `.t` file introduces a scope named after the file.
4. `module.t` defines its containing directory scope and contributes no name.
5. Scope names prefix child scopes or terminal values.
6. `use` makes a published path available; a scope path includes its published
   subtree without flattening it.
7. `is` may bind either a scope or a value to a local name.
8. Declarations are private unless published with `pub`.
9. Publication crosses exactly one module boundary.
10. Publishing a scope preserves the visibility of everything inside it.

These rules make module dependencies concise while retaining explicit
encapsulation at every level of the source tree.

## The language module

`lang` is a compiler-provided special module. Unlike an application or library
module, it can provide grammar, constructs, typing rules, compiler guarantees,
scopes, and values. Its contents are introduced directly into the source file's
main scope rather than remaining qualified by `lang`.

Every source file selects an immutable language revision explicitly:

```topal
use lang topal-r3
```

The compiler parses and checks the rest of the file according to revision 3.
This prevents a later revision from changing the meaning of existing source,
including by treating one of its identifiers as a newly introduced structural
word.

A language revision may contain variants represented by nested scopes:

```topal
use lang topal-r3 realtime
```

The `realtime` variant may add constructs that require the compiler to prove
execution-time guarantees for selected algorithms. Other variants may restrict
the language for smaller processors or execution environments. A variant
refines a particular revision; it does not silently track newer revisions.

Language selection has stricter rules than an ordinary `use`:

1. Every source file, including `module.t`, selects a language.
2. Selection is the first non-comment declaration in the file.
3. A file contains exactly one language selection.
4. An omitted selection is an error rather than an implicit request for the
   latest revision.
5. The revision and variant are recorded as part of the compiled module.

Files in one package may select different language revisions or variants. The
compiler checks that their published interfaces agree on representations and
semantics needed for interoperability.

### Bootstrap syntax

The compiler must recognize language selection before it knows which language
grammar to apply. A small, stable bootstrap syntax therefore recognizes line
boundaries, `# ` comments, and the `use lang` declaration with its revision and
optional variant path. After that declaration, the selected language defines
the remainder of the file.

For example:

```topal
# Compiled using the realtime variant of revision 3.
use lang topal-r3 realtime

use logger
```

The bootstrap syntax is fixed across language revisions so that every compiler
can determine which revision is being requested.
