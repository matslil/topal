# Modules, scopes, and visibility

Topal source layout defines a hierarchy of scopes. A source package contains
one root scope and may expose that scope, or scopes nested within it, as
libraries and applications. Directories form nested scopes, and ordinary
source files form scopes within their containing directory. Functions and other
values are the terminal names in this hierarchy.

This model keeps the source tree and canonical names aligned while allowing a
module to expose an interface that does not mirror its implementation layout.

## Source layout and canonical names

Except for the special files `module.t`, `package.t`, `library.t`, and
`application.t`, each source filename contributes one component to the scope
path. Each special file describes its containing directory or a view of that
directory and contributes no component of its own. The `.t` suffix is not part
of an ordinary file's component. Given:

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

`module.t` is an exception to the ordinary filename rule. It represents its
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

1. The source-package root is the top-level scope.
2. A directory introduces a nested scope.
3. An ordinary `.t` file introduces a scope named after the file.
4. The special files `module.t`, `package.t`, `library.t`, and `application.t`
   contribute no scope-path component.
5. Scope names prefix child scopes or terminal values.
6. `use` makes a published path available; a scope path includes its published
   subtree without flattening it.
7. `is` may bind either a scope or a value to a local name.
8. Declarations are private unless published with `pub`.
9. Publication crosses exactly one module boundary.
10. Publishing a scope preserves the visibility of everything inside it.

These rules make module dependencies concise while retaining explicit
encapsulation at every level of the source tree.

The four exact filenames are reserved by the language. They remain visible in
ordinary directory listings because they describe important build and interface
boundaries. Other filenames are not reserved merely because they begin with a
period; filesystem conventions such as hidden files are kept separate from
Topal's language-level source conventions.

## The language module

`lang` is a compiler-provided special module. Unlike an application or library
module, it can provide grammar, constructs, typing rules, compiler guarantees,
scopes, and values. Its contents are introduced directly into the source file's
main scope rather than remaining qualified by `lang`.

Static [introspection](introspection.md) is the deliberate exception to that
last rule. Introspection-specific operations and descriptor types remain
available through the qualified `lang` scope, as in `lang view Person` and
`lang TypeView`. This keeps inspection of language objects visibly distinct
from ordinary application and data selection. Selecting a language revision
still introduces that revision's ordinary source vocabulary directly.

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

## Source packages

A source package is the unit obtained by the package manager. Its source root
contains a `package.t` file. Like every Topal source file, `package.t` begins by
selecting a language revision:

```topal
use lang topal-r3

use package se.example.numerics version 2.4
use package org.example.rendering version 7.2 features (
  gpu
  png
)

package is se.example.calculator
version is 5.3.1
```

Only `use lang` is part of the bootstrap syntax. Once it has selected an
immutable language revision, that revision defines the grammar and meaning of
`use package`, version values, feature lists, and every other declaration in
`package.t`. Package syntax may consequently evolve between language revisions
without enlarging the fixed bootstrap grammar.

`package.t` is an ordinary Topal program subject to an additional static
requirement: every declaration and expression in it must be evaluable at
compile and package-resolution time. Evaluation is deterministic and cannot
perform effects or depend on code from a package being selected. In particular,
dependency discovery cannot require first fetching or compiling that
dependency.

The file defines classified values with meanings known to the package manager,
including at least the package identity and package version. A reverse-DNS name
such as `se.example.calculator` provides the canonical package identity. The
package manager treats it as a structured identifier; authentication of its
publisher is a separate registry concern.

`use package` selects an external source package. Its `version` states a
compatible requirement, while a package lock records the exact resolved release
for reproducible builds. Features are static selections recorded as part of the
resolution. The selected language revision may define additional package
metadata and richer static computations, provided dependency discovery remains
possible before external package code is available.

## Libraries and applications

A source package is a distribution and need not correspond one-to-one with a
library or application. A directory may contain any of `module.t`, `library.t`,
and `application.t` in parallel. The files have distinct roles:

- `module.t` constructs the shared implementation scope of its directory.
- `library.t` defines a versioned, linkable view of that scope.
- `application.t` defines a versioned, executable view of that scope.
- `package.t`, at the source root, describes the distribution and its external
  package dependencies.

For example, a small calculator may be distributed as both a library and an
application without dividing its implementation into artificial directories:

```text
calculator/
├── package.t
├── module.t
├── library.t
└── application.t
```

The shared implementation belongs in `module.t`:

```topal
# module.t
use lang topal-r3

calculate is fn ( expression : String ) -> Result Number
  implementation
```

The two artifact files select different external views and may assign them
independent versions:

```topal
# library.t
use lang topal-r3

version is 2.1.0
pub calculate
```

```topal
# application.t
use lang topal-r3

version is 4.3.2

start is fn ( arguments : CommandArguments ) -> Result Completed
  selected is arguments expression
  print calculate selected
  Completed
```

Both artifact files can resolve private names in the shared directory scope.
Publishing `calculate` from `library.t` does not make it part of the application
interface. The root task in `application.t` may nevertheless call `calculate`
directly because both are compiled from the same source module.

Each artifact file has a local facade scope layered over the directory scope.
Facade-local metadata and aliases do not merge into the implementation or the
other facade. The two `version` declarations above therefore do not conflict.
In an artifact facade, `pub` crosses the artifact boundary:

- In `library.t`, it adds a name to the linkable library interface.
- In `application.t`, it adds a name to the application interface, such as an
  externally available command, configuration value, required capability, or
  service endpoint. The root task and its platform lifecycle handlers are
  established by `application.t` itself and do not require publication.

`application.t` is the implicit [root task](tasks.md) of the executable. Its
selected language variant defines the available lifecycle and platform-event
handlers. Topal constructs this task and supplies platform startup values, such
as command arguments, directly to its `start` handler; no separate application
type or `main` algorithm is required.

In ordinary source files and `module.t`, publication retains its normal
one-module-boundary meaning. `package.t` supplies package metadata rather than
an artifact interface.

A source package may contain multiple artifacts. When it does, directories
provide their names and boundaries naturally:

```text
package.t
common/
└── module.t
vector/
├── module.t
└── library.t
viewer/
├── module.t
└── application.t
```

An artifact's canonical identity combines its source-package identity, artifact
kind, and path. The kind distinguishes a root library from a root application
when both are present; the path distinguishes multiple artifacts of the same
kind, as in the `vector` library of `se.example.geometry`. A local binding may
give the canonical identity a shorter name without changing what compiled
dependencies record.

## Package and artifact versions

Package and artifact versions describe different things. A package version
identifies a fetched source distribution. A library version describes the
compatibility of a linkable interface, and an application version describes the
evolution of an executable interface. Changing package documentation or one
application need not change the version of an unaffected library in the same
package.

Within a stable major version, minor versions accumulate functionality and
patch versions preserve the interface. A compiled dependency on library
version `3.7.4` is compatible with an available library exactly when:

```text
available.major = 3
available.minor >= 7
available.patch is unrestricted
```

A different major version is an explicit compatibility boundary and is
rejected even if the particular names used by a client still exist. A lower
minor version is rejected because required functionality may be absent. A
different patch version does not by itself cause a compilation error. Exact
release selection remains in the package lock.

The compiler may record the minor version in which each published declaration
was introduced and infer the actual minimum minor version required by a client.
This avoids requiring a later minor merely because it happened to be installed
when the client was compiled.

These rules guarantee interface availability, not behavioral correctness. A
patch may contain a faulty implementation without changing its types or
published interface. Tests in the producing package, supplemented by tests in
dependent packages, establish the behavioral confidence that version checking
cannot provide.

Features form part of an artifact selection. Features intended for dependency
unification must be additive: enabling another feature may add interface
members but cannot remove or alter the base interface. Requests for such
features can be combined by union. A future non-additive feature mechanism
would instead have to treat different selections as distinct artifact variants.

## Licenses and copyright

License and copyright information is static source metadata. Every function has
an effective license and effective copyright information in each artifact
context in which it is compiled. The compiler retains that provenance through
specialization, inlining, code generation, and linking so that the package
builder can check the obligations of the resulting source and binary artifacts.

Topal recognizes only licenses and exceptions identified by the supported SPDX
License List. A license value uses the canonical SPDX identifier, including its
version distinction:

```topal
license is Apache-2.0
```

SPDX license expressions represent a choice of terms, simultaneous terms, or a
standard exception:

```topal
license is MIT or Apache-2.0
license is MIT and BSD-3-Clause
license is GPL-2.0-or-later with Classpath-exception-2.0
```

The selected language revision defines the Topal surface syntax, while the
identifiers and the meanings of `or`, `and`, and `with` follow the supported
SPDX data. An unknown identifier, custom `LicenseRef`, or unrecognized
exception is an error. Restricting declarations to this finite vocabulary lets
the toolchain attach reviewed compatibility and obligation rules to every
accepted license.

Copyright metadata identifies one or more holders and the years attributed to
their work:

```topal
copyright is (
  holder "Example AB"
  years 2024 through 2026
)
```

Several notices remain distinct when several holders contributed:

```topal
copyrights are (
  copyright holder "Example AB" years 2024 through 2026
  copyright holder "Alice Smith" years 2025
)
```

The compiler checks and propagates these declarations but does not establish
that a declared party owns the copyright or has authority to select a license.
Those are assertions made by the package publisher.

### Mandatory package defaults

The source root's `package.t` provides the final license and copyright defaults
for the complete package:

```topal
use lang topal-r3

package is se.example.calculator
version is 5.3.1
license is Apache-2.0

copyright is (
  holder "Example AB"
  years 2024 through 2026
)
```

Both defaults are mandatory. A package without either one is invalid. This
makes metadata resolution total without requiring every source file to repeat
package-wide information.

### Metadata inheritance

A function may declare license or copyright metadata directly. Otherwise it
inherits each missing item independently from the nearest applicable source
context:

1. The function declaration.
2. Its ordinary source file.
3. The containing directory's applicable `application.t`, `library.t`, or
   `module.t` declaration.
4. Successive parent directories, from nearest to farthest.
5. The mandatory declaration in the source root's `package.t`.

A declaration may provide a license while inheriting copyright, or provide
copyright while inheriting a license. The two searches are independent.

An ordinary source file can establish defaults for its functions:

```topal
use lang topal-r3

license is MIT
copyright is (
  holder "Alice Smith"
  years 2025
)

parse is fn ...
tokenize is fn ...
```

Both functions inherit the file declarations unless they provide more specific
metadata. Function-specific metadata supports a file containing code with
different provenance, for example an algorithm adapted from another project.
The precise modifier syntax remains part of the language revision, but the
metadata belongs to the function declaration rather than becoming an input or
runtime value.

At each directory, `module.t` describes defaults intrinsic to the shared
implementation:

```topal
# parser/module.t
use lang topal-r3

license is BSD-3-Clause
copyright is (
  holder "Parser contributors"
  years 2022 through 2026
)
```

These defaults also apply to descendants that have no more specific
declaration.

### Artifact contexts

`library.t` and `application.t` may establish different contextual defaults for
the same directory. A declaration in `library.t` applies while constructing
that library interface and implementation; a declaration in `application.t`
applies while constructing that application:

```topal
# library.t
use lang topal-r3

version is 2.1.0
license is LGPL-3.0-only
pub calculate
```

```topal
# application.t
use lang topal-r3

version is 4.3.2
license is GPL-3.0-only

start is fn ( arguments : CommandArguments ) -> Result Completed
  launch arguments
  Completed
```

At the same directory, the facade matching the artifact being constructed is
more specific than `module.t`. The contextual declaration affects only
functions that reach it without function- or file-specific metadata. It does
not erase explicit provenance attached to incorporated or imported code.

Consequently, one shared function can have different effective licenses in the
library and application contexts when its nearest applicable declarations
offer it under those different terms. A function with an explicit license
retains that license in both artifacts. The compiler rejects an artifact whose
selected terms cannot satisfy the effective licenses of all included
functions.

Copyright normally describes provenance rather than an artifact choice. Shared
functions therefore retain the same holders in the library and application
unless more specific source declarations state otherwise. Copyright declared
in an artifact facade supplies a default for code belonging to that context; it
does not reassign explicitly attributed shared code.

### Composition and obligations

When functions are combined, copyright notices accumulate rather than replace
one another. License expressions are combined according to their SPDX choices
and the toolchain's compatibility rules. The result need not be a single new
license: components retain their provenance even when one license determines
the permitted terms of a combined artifact.

The package builder derives presentation obligations from the licenses and the
kind of output being produced. It can require and generate, as appropriate:

- Copyright and attribution notices in distributed source.
- License texts and notice files in source or binary distributions.
- Acknowledgements in generated documentation.
- License and attribution information exposed by an application at runtime.

The compiler verifies that every included function has resolved metadata and
that transformations preserve it. The package builder verifies that every
derived obligation has a declared output location. Missing notices,
incompatible terms, or an absent documentation or runtime presentation surface
are build errors.

Copyright years are source metadata, not build timestamps. Rebuilding an
artifact in a later year does not modify its notices. A tool may compare years
with version-control history and warn about a possible stale declaration, but
such evidence does not prove ownership or the legally correct year.
