# Modules, scopes, and visibility

Topal source layout defines a hierarchy of scopes. A source package constructs
its root scope, and each child module constructs a nested scope. A package may
expose its root scope, or scopes nested within it, as libraries and
applications. Ordinary source files form scopes within their containing
constructed context. Functions and other values are the terminal names in this
hierarchy.

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
├── package.t
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
including a function. Name resolution walks through scope components until
it reaches a value. Any remaining expressions are applied to that value:

```topal
logger emit-logs err Message
```

Here `logger` and `emit-logs` resolve scopes, `err` resolves a function, and
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
scope, and then applies the resulting function to `Message`. Bindings can
shorten a path or give it a name appropriate to the current module:

```topal
diagnostics is logger emit-logs
diagnostics warning Message
```

Aliasing a scope neither copies its contents nor changes their visibility.

## Function name resolution

An ordinary function does not automatically traverse from its lexical body
into its source-file or directory namespace. Unqualified names resolve only
parameters and bindings in the function's nested lexical scopes. Namespace
members use an explicit qualified path.

When a qualified path resolves to an overloaded function name, declarations in
that namespace are tested in source order and the first applicable input header
is selected. `use` makes the path visible without flattening it, so overloads
from separately used scopes never acquire a combined import order. Binding a
scope to a local alias likewise preserves its namespace and overload order.

The `@` prefix provides the separate, deliberately narrow access to a member of
the function's current constructed `Package` or `Module`:

```topal
calculate is fn ( input : Number ) -> Number
  input * @ scale
```

The member may be a constructor argument or a declaration introduced directly
by `package.t` or `module.t`. It does not select an arbitrary ordinary-file
binding or search a parent or calling namespace. Context construction is
defined in [constructed package and module contexts](contexts.md).

## Private by default

Every declaration in a source file is private by default. `pub` makes a name
visible across its nearest enclosing constructed-context boundary. For
functions, `pub` occupies the modifier position used by `static`:

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

A published declaration must not expose a private type through its interface.
In particular, every error vocabulary explicitly named by a function's
`Result` is part of its contract. Publishing the function therefore requires
each vocabulary to be published through a namespace reachable by consumers.
The vocabularies may be defined and published independently of the function
and need not share its namespace:

```topal
use shared-error

read-count is fn pub (
  path : Path
) -> Result ( Integer, shared-error FileErrorCode )
  body
```

Here `read-count` remains usable only while `shared-error FileErrorCode` is
part of the published dependency interface. The function's publication does
not implicitly relocate or republish that type.

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

## Constructed directory contexts

Every directory has exactly one constructed context. At the source root,
`package.t` declares `Package`. In every constructed child directory,
`module.t` declares `Module`. `Package` is a superset of `Module`: it performs
the same implementation-scope construction and additionally declares package
identity, version, dependencies, and other package-manager metadata.

Consequently, `package.t` and `module.t` are mutually exclusive within a
directory, and a source-root `module.t` is invalid. Both special files represent
their containing directory and add no component to its scope path. Definitions
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

Consequently, `module.t` is both the constructor of a child directory scope and
the place where that directory assembles its public interface. At the root,
`package.t` has the same responsibilities in addition to its package-specific
ones. Either may declare members directly, publish selected members from child
files, rename them, or publish selected child scopes. The physical contents of
a directory do not by themselves determine its external interface.

## Resolution and visibility rules

The module model follows these rules:

1. `Package` constructs the source-package root scope.
2. `Module` constructs each child directory scope.
3. An ordinary `.t` file introduces a scope named after the file.
4. The special files `module.t`, `package.t`, `library.t`, and `application.t`
   contribute no scope-path component.
5. Scope names prefix child scopes or terminal values.
6. `use` makes a published path available; a scope path includes its published
   subtree without flattening it.
7. `is` may bind either a scope or a value to a local name.
8. Declarations are private unless published with `pub`.
9. Publication crosses exactly one constructed-context boundary.
10. Publishing a scope preserves the visibility of everything inside it.
11. A directory contains either `package.t` at the source root or `module.t`
    below it, never both.
12. Functions access members of their constructed context explicitly with `@`;
    they do not traverse automatically into file or parent directory scopes.

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
from ordinary application and data selection. Compiler diagnostic controls are
also deliberately qualified, as in `lang disable-warning unverified-law`;
their lexical effect does not make them runtime module state. Selecting a
language revision still introduces that revision's ordinary source vocabulary
directly.

Every source file begins by selecting an immutable language version explicitly:

```topal
use lang topal (
  version is v1.5
)
```

The `v` prefix constructs a `Version` value. `v1.5` is the abbreviated spelling
of `v1.5.0-0`; the complete form contains natural-number major, minor, patch,
and build components. Versions are ordered lexicographically by those four
components.

The selection establishes a language context from its occurrence forward.
Another top-level selection may begin a region using another version:

```topal
use lang topal (
  version is v1.5
)

current-function is ...

use lang topal (
  version is v1.4
)

compatible-function is ...
```

Every complete top-level declaration is parsed and checked under the exact
context active at its declaration. A selection cannot occur inside a
declaration, and mutually recursive declaration groups cannot cross a language
boundary. References across regions are checked like published objects
exchanged by files selecting different language versions.

After the mandatory initial selection, a source file may construct and name an
inactive language context:

```topal
legacy is use lang topal (
  version is v1.0,
  features is ()
)
```

Binding the context does not change how following source is parsed. A later
top-level `use legacy` activates that complete context from its occurrence
forward. Both construction and activation are static, activation occurs only
between complete top-level declarations, and an activation replaces rather
than merges with the previous language context.

A selected version may accept optional features as an ordinary static
constructor argument:

```topal
use lang topal (
  version is v1.5,
  features is ( realtime )
)
```

`features` has no privileged language meaning; it is the conventional argument
through which the selected language implementation enables grammar, structural
words, compiler guarantees, and compiler-provided objects. Feature identities
and their effects are defined by that immutable language version. Enabled
vocabulary normally enters the root scope. Static introspection remains
deliberately qualified under `lang`, and an implementation may apply the same
exceptional policy when qualification communicates an important semantic
boundary.

Every language `use` supplies a complete constructor argument set. A later
selection starts a new context from its occurrence forward rather than
incrementally adding to the earlier feature set. Activation reports a conflict
with an already visible root binding rather than silently shadowing either
meaning.

For example, a file can visibly separate production declarations from tests:

```topal
use lang topal (
  version is v1.5
)

eligible is fn ( user : User ) -> Bool
  ...

use lang topal (
  version is v1.5,
  features is ( testing )
)

eligible path-coverage
  user-a, () -> true
```

Language selection has stricter rules than an ordinary module `use`:

1. Every source file, including `module.t`, selects a language.
2. The initial selection is the first non-comment declaration in the file.
3. Later language selections occur only between complete top-level
   declarations.
4. Each selection affects only the source which follows it.
5. An omitted initial selection is an error rather than an implicit request for the
   latest revision.
6. Every compiled declaration records its exact constructed language context.

Files in one package may construct different language contexts. The
compiler checks that their published interfaces agree on representations and
semantics needed for interoperability.

### Bootstrap syntax

The compiler must recognize the initial language construction before it knows
which language grammar to apply. A small, stable bootstrap syntax therefore
recognizes line boundaries, `# ` comments, `use lang topal`, and its
comma-separated constructor record made from identifier associations and basic
static object literals. Its general contextual type is `Record ( Identifier,
Object )`. The bootstrap parser delimits the record and uses its `version`
association to locate the language implementation; that implementation defines
the record's complete names, types, and meaning. After that declaration, the
selected language defines how later language constructions and all ordinary
source are parsed.

The bootstrap syntax knows no capability vocabulary or capability-combination
rules. Atomic capabilities, their classified object kinds, their promises, and
the semantics of capability expressions belong entirely to the selected Topal
version.

For example:

```topal
# Begin with Topal 1.5 and its realtime feature.
use lang topal (
  version is v1.5,
  features is ( realtime )
)

use logger
```

The initial bootstrap syntax is fixed across language versions so that every
compiler can determine which version is being requested. A compiler which does
not implement that version rejects the file at this boundary.

## Source packages

A source package is the unit obtained by the package manager. Its source root
contains a `package.t` file. Like every Topal source file, `package.t` begins by
selecting a language revision:

```topal
use lang topal (
  version is v1.5
)

Package (
  version : Version,
  features : Set CalculatorFeature default ()
)

use package se.example.numerics (
  version is v2.4
)
use package org.example.rendering (
  version is v7.2,
  features is ( gpu, png )
)

package identity is "se.example.calculator"
package version is v5.3.1
package status is maintained
package description is "Exact numerical calculations"
```

The initial `use lang topal` uses the same construction-record syntax as every
later construction. Its restricted record grammar is part of the bootstrap
syntax. Once it has selected an immutable language revision, that revision
defines the grammar and meaning of `Package`, `use package`, version values,
constructor arguments, and every other declaration in `package.t`. Package
syntax may consequently evolve between language revisions without otherwise
enlarging the fixed bootstrap grammar.

`package.t` constructs the root implementation scope and is a semantic superset
of `module.t`. Its top-level declarations and expressions are subject to an
additional static requirement: they must be evaluable at compile and
package-resolution time. A constructor argument used by that evaluation must
therefore be static; a runtime argument may be selected only from inside a
runtime body. Static evaluation is deterministic and cannot perform effects or
depend on code from a package being selected. In particular, dependency
discovery cannot require first fetching or compiling that dependency.

The compiler-provided `package` metadata namespace distinguishes authoritative
package properties from constructor arguments and ordinary bindings. Only
`package.t` may declare its fields, and every declaration is static and
immutable:

```topal
package identity is "se.example.calculator"
package version is v5.3.1
```

A string containing a canonical reverse-DNS name such as
`"se.example.calculator"` provides the package identity. Authentication of its
publisher is a separate registry concern. The selected language revision
defines the metadata schema and rejects unknown or duplicate fields. `package`
is a structural metadata namespace, not a newly constructed ordinary variable
and not part of the package's published value interface.

### Package metadata

Package metadata has the following fields:

```text
identity       String
version        Version
status         PackageStatus
description    String
license        String
copyrights     List Copyright

repository     URI
homepage       URI
documentation  URI
issues         URI
readme         URI
maintainers    List ( String, Optional URI )
keywords       List String
categories     List String
funding        List URI
```

`PackageStatus` is one of `development`, `maintained`, and `archived`.
`description` is free text. `license` contains either `private` or an SPDX
license expression which the compiler validates. Each maintainer tuple contains
a name and an optional contact URI, such as a `mailto` URI or homepage.
Keywords and categories remain separate strings rather than being encoded into
one delimiter-dependent string.

The mandatory fields depend on the package status:

| Field | `development` | `maintained` | `archived` |
| --- | --- | --- | --- |
| `identity` | required | required | required |
| `version` | required | required | required |
| `status` | required | required | required |
| `description` | required | required | required |
| `license` | required | required | required |
| `copyrights` | required | required | required |
| `maintainers` | required | required | optional |
| `repository` | required | required | required |
| `issues` | optional | required | optional |
| `documentation` | optional | required | optional |
| `homepage` | optional | optional | optional |
| `readme` | optional | optional | optional |
| `keywords` | optional | optional | optional |
| `categories` | optional | optional | optional |
| `funding` | optional | optional | optional |

A required list must be nonempty. A `development` package identifies who is
responsible and where development occurs, but need not yet have complete
documentation or an issue tracker. A `maintained` package additionally provides
both. An `archived` package retains a durable source location, but does not
promise a responsive maintainer or issue tracker.

Package tools discover a conventional README, such as `README.md`, in the
package source without requiring metadata. The optional `readme` field selects
a nonstandard package-relative or external URI instead. Accordingly, `URI`
accepts both absolute URIs and relative URI references in package metadata.

Status records the publisher's assertion when a particular package release is
created. A registry may separately retain mutable current-project status,
initially copied from the newest release, so archiving a project does not
require publishing an otherwise artificial release.

For example:

```topal
package identity is "se.example.calculator"
package version is v5.3.1
package status is maintained
package description is "Exact numerical calculations"
package license is "Apache-2.0"
package copyrights is (
  (
    holder is "Example AB",
    years is ( 2024 .. 2026 )
  )
)
package repository is URI "https://github.com/example/calculator"
package documentation is URI "https://docs.example.com/calculator"
package issues is URI "https://github.com/example/calculator/issues"
package maintainers is (
  ( "Alice Smith", URI "mailto:alice@example.com" )
)
package keywords is ( "calculator", "exact-arithmetic" )
package categories is ( "mathematics" )
```

At the top level of `package.t`, an unqualified `version` resolves the
constructor argument while `package version` resolves authoritative metadata.
A function has neither name implicitly and selects them through its constructed
context:

```topal
requested-version is fn () -> Version
  @ version

actual-version is fn () -> Version
  @ package version
```

`use package` selects and constructs an external source package. Its `version`
constructor argument states a compatible requirement, while `package version`
is the authoritative version declared by the selected release. A package lock
records the exact resolved release and all interface- or dependency-shaping
constructor arguments for reproducible builds. `features` is merely a
conventional argument name; package code may use its statically known value to
select dependencies, declarations, or published members. The selected language
revision may define additional package metadata and richer static computations,
provided dependency discovery remains possible before external package code is
available.

## Libraries and applications

A source package is a distribution and need not correspond one-to-one with a
library or application. The source root may contain `package.t`, `library.t`,
and `application.t` in parallel. A child directory may contain `module.t`,
`library.t`, and `application.t` in parallel. The files have distinct roles:

- `package.t` constructs the shared root implementation scope and defines the
  distribution and its external package dependencies.
- `module.t` constructs the shared implementation scope of a child directory.
- `library.t` defines a versioned, linkable view of that scope.
- `application.t` defines a versioned, executable view of that scope.

For example, a small calculator may be distributed as both a library and an
application without dividing its implementation into artificial directories:

```text
calculator/
├── package.t
├── library.t
└── application.t
```

The shared root implementation belongs in `package.t`:

```topal
# package.t
use lang topal (
  version is v1.5
)

Package (
  version : Version
)

calculate is fn ( expression : String ) -> Result ( Number, ModuleErrorCode )
  implementation
```

The two artifact files select different external views and may assign them
independent versions:

```topal
# library.t
use lang topal (
  version is v1.5
)

library version is v2.1.0
calculate is @ calculate
pub calculate
```

```topal
# application.t
use lang topal (
  version is v1.5
)

application version is v4.3.2

start is fn (
  arguments : CommandArguments,
  environment : Map ( String, String )
) -> Result ( Completed, ModuleErrorCode )
  selected is arguments expression
  print @ calculate selected
  Completed
```

Both artifact files can explicitly select private names in the shared
constructed context with `@`.
Publishing `calculate` from `library.t` does not make it part of the application
interface. The root task in `application.t` may nevertheless call `calculate`
as `@ calculate` because both are compiled in the same source context.

Each artifact file has a local facade scope layered over the directory scope.
The compiler provides a `library` or `application` metadata namespace in the
matching facade. Only its owning artifact file may declare its fields, which
are static, immutable, and defined by the selected language revision.
Facade-local metadata and aliases do not merge into the implementation or the
other facade. `package version`, `library version`, and `application version`
therefore remain distinct.

At a facade's top level its metadata namespace is directly available. A
function has no implicit access to that outer scope and must select the complete
qualified path through `@`:

```topal
reported-version is fn () -> Version
  @ application version
```

The same rule applies in `package.t`: a function reads `@ package version`, not
`package version`. A shared function defined outside a facade cannot select
facade metadata which is absent from its defining context.

In an artifact facade, `pub` crosses the artifact boundary:

- In `library.t`, it adds a name to the linkable library interface.
- In `application.t`, it adds a name to the application interface, such as an
  externally available command, configuration value, required capability, or
  service endpoint. The root task and its platform lifecycle handlers are
  established by `application.t` itself and do not require publication.

`application.t` is the implicit [root task](tasks.md) of the executable. Its
constructed language context defines the available lifecycle and platform-event
handlers. Topal constructs this task and supplies platform startup values, such
as command arguments and environment variables, directly to its `start`
handler; no separate application type or `main` function is required.

In ordinary source files, `module.t`, and `package.t`, publication retains its
normal one-context-boundary meaning. Package publication and package metadata
remain distinct from an artifact interface.

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
version `v3.7.4` is compatible with an available library exactly when:

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

Constructor arguments form part of an artifact selection when they affect its
interface. Different argument records construct distinct variants; Topal does not
implicitly union an argument named `features` or require it to be additive.
Code which wants an additive feature convention declares and implements that
policy explicitly.

## Generic intermediate code

A compiled Topal library artifact may contain both final machine code and typed
generic intermediate code. Concrete exported implementations can be emitted as
machine code immediately. Exported generic implementations normally retain:

- their typed intermediate body;
- symbolic type and construction parameters;
- capability and implementation evidence;
- result error-vocabulary parameters and bounds;
- effect rows and resource-identity relationships;
- layout and static-value dependencies; and
- provenance required for diagnostics, licensing, and reproducibility.

Final application compilation substitutes the selected concrete types,
callbacks, error vocabularies, identities, layouts, and evidence. It then
specializes the generic body, emits machine code, and deduplicates
definitionally identical specializations. Incremental caches may retain
generated specializations, but cache contents do not change program semantics.

Finite dynamic implementation alternatives may each receive a useful
specialization. When identity is deliberately erased or specialization would
not distinguish the remaining alternatives, the compiler emits a shared
fallback using the conservative union of effects and other evidence required
by every possible value.

Generic intermediate code is compiler-versioned implementation material, not a
permanent public machine ABI. A compiler which cannot consume its version
rebuilds it from a compatible source artifact or reports that the compiled
dependency must be rebuilt. Published Topal types, interfaces, capabilities,
and semantic contracts remain the compatibility boundary.

A producer may intentionally conceal an implementation by publishing concrete
shared code and a sufficient opaque contract instead of its generic body. This
forgoes body-derived specialization and may require conservative effects,
layouts, or resource relationships. Foreign exports always require a concrete
ABI; unresolved Topal generic intermediate code cannot cross that boundary.

## Licenses and copyright

License and copyright information is static source metadata. Every function has
an effective license and effective copyright information in each artifact
context in which it is compiled. The compiler retains that provenance through
specialization, inlining, code generation, and linking so that the package
builder can check the obligations of the resulting source and binary artifacts.

Except for the special package value `private`, Topal recognizes only licenses
and exceptions identified by the supported SPDX License List. A license value
is a string containing the canonical SPDX identifier, including its version
distinction:

```topal
license is "Apache-2.0"
```

SPDX license expressions represent a choice of terms, simultaneous terms, or a
standard exception:

```topal
license is "MIT OR Apache-2.0"
license is "MIT AND BSD-3-Clause"
license is "GPL-2.0-or-later WITH Classpath-exception-2.0"
```

The compiler parses and validates the string according to the supported SPDX
data. An unknown identifier, custom `LicenseRef`, or unrecognized exception is
an error. Restricting declarations to this finite vocabulary lets the toolchain
attach reviewed compatibility and obligation rules to every accepted license.
`"private"` states that no distribution license is granted; an artifact which
requires such permission cannot be distributed.

Copyright metadata identifies one or more holders and the years attributed to
their work:

```topal
copyrights is (
  (
    holder is "Example AB",
    years is ( 2024 .. 2026 )
  )
)
```

Several notices remain distinct when several holders contributed:

```topal
copyrights is (
  (
    holder is "Example AB",
    years is ( 2024 .. 2026 )
  ),
  (
    holder is "Alice Smith",
    years is ( 2025 )
  )
)
```

`Copyright` has a `holder : String` field and a
`years : List (Range Year)` field. A singleton year is accepted where a
singleton range is expected. Keeping years as a list permits gaps without
claiming one continuous range, and `..` is Topal's ordinary inclusive range
syntax.

The compiler checks and propagates these declarations but does not establish
that a declared party owns the copyright or has authority to select a license.
Those are assertions made by the package publisher.

### Mandatory package defaults

The source root's `package.t` provides the final license and copyright defaults
for the complete package:

```topal
use lang topal (
  version is v1.5
)

package identity is "se.example.calculator"
package version is v5.3.1
package status is maintained
package description is "Exact numerical calculations"
package license is "Apache-2.0"
package copyrights is (
  (
    holder is "Example AB",
    years is ( 2024 .. 2026 )
  )
)
package repository is URI "https://github.com/example/calculator"
package documentation is URI "https://docs.example.com/calculator"
package issues is URI "https://github.com/example/calculator/issues"
package maintainers is (
  ( "Alice Smith", URI "mailto:alice@example.com" )
)
```

The applicable status-dependent package fields are mandatory. In particular,
`license` and the nonempty `copyrights` list provide final defaults for source
provenance. This makes metadata resolution total without requiring every source
file to repeat package-wide information.

### Metadata inheritance

A function may declare license or copyrights metadata directly. Otherwise it
inherits each missing item independently from the nearest applicable source
context:

1. The function declaration.
2. Its ordinary source file.
3. The containing directory's applicable `application.t`, `library.t`, or
   `module.t` declaration.
4. Successive parent directories, from nearest to farthest.
5. The mandatory declaration in the source root's `package.t`.

A declaration may provide a license while inheriting copyrights, or provide
copyrights while inheriting a license. The two searches are independent.

An ordinary source file can establish defaults for its functions:

```topal
use lang topal (
  version is v1.5
)

license is "MIT"
copyrights is (
  (
    holder is "Alice Smith",
    years is ( 2025 )
  )
)

parse is fn ...
tokenize is fn ...
```

Both functions inherit the file declarations unless they provide more specific
metadata. Function-specific metadata supports a file containing code with
different provenance, for example a function adapted from another project.
The precise modifier syntax remains part of the language revision, but the
metadata belongs to the function declaration rather than becoming an input or
runtime value.

At each directory, `module.t` describes defaults intrinsic to the shared
implementation:

```topal
# parser/module.t
use lang topal (
  version is v1.5
)

license is "BSD-3-Clause"
copyrights is (
  (
    holder is "Parser contributors",
    years is ( 2022 .. 2026 )
  )
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
use lang topal (
  version is v1.5
)

library version is v2.1.0
library license is "LGPL-3.0-only"
pub calculate
```

```topal
# application.t
use lang topal (
  version is v1.5
)

application version is v4.3.2
application license is "GPL-3.0-only"

start is fn (
  arguments : CommandArguments,
  environment : Map ( String, String )
) -> Result ( Completed, ModuleErrorCode )
  launch ( arguments, environment )
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
unless more specific source declarations state otherwise. Copyrights declared
in an artifact facade supply a default for code belonging to that context; they
do not reassign explicitly attributed shared code.

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
