# Constructed package and module contexts

Topal packages and modules are immutable constructed namespace values. Their
constructor arguments provide explicitly declared contextual values without
process-global identity or shared mutable state. A function selects a member of
its defining context with the `@` prefix.

Every source directory has exactly one context constructor:

- the source root uses `Package`, declared by `package.t`; and
- every constructed child directory uses `Module`, declared by `module.t`.

`Package` is a semantic superset of `Module`. It constructs the root
implementation scope like a module and additionally supplies the package
identity, version, dependency declarations, and other package-manager metadata.
A source-root `module.t` is consequently invalid.

Ordinary source files, `library.t`, and `application.t` do not introduce
separate contexts. Functions declared in them use the `Package` or `Module` of
their directory.

## Constructor declarations

Constructor declarations classify their arguments rather than defining an
ordinary function:

```topal
# logger/module.t

Module (
  destination : LogDestination,
  minimum-level : LogLevel default Info
)

write is fn ( message : Message ) -> Unit
  if message level >= @ minimum-level
    (@ destination) send message
```

At the source root the corresponding declaration is `Package`:

```topal
# package.t

Package (
  version : Version,
  features : Set PackageFeature default (),
  diagnostics : Diagnostics
)

package identity is se.example.calculator
package version is v5.3.1
```

The capitalized structural words distinguish context construction from `fn`
declarations and ordinary value bindings. Their source position is not
semantic. After the mandatory initial language selection, the compiler collects
the complete file's context constructor and recognized metadata declarations
before checking static definitions and dependency evaluation. A `package.t`
must contain exactly one `Package` declaration, and a `module.t` exactly one
`Module` declaration, but neither must precede the file's other declarations.

Package metadata and applicable module/source defaults may likewise be
interleaved with dependencies and ordinary declarations. The compiler rejects
duplicates and checks mandatory metadata after collection. Ordinary `use`
visibility and overload source order are unchanged; moving a structural
constructor or metadata declaration between them does not reorder those
declarations. Static value and dependency relationships must still be acyclic.

The file's initial language selection defines the `Package` or `Module`
constructor and metadata schema for the complete file. A later language
selection may govern subsequent ordinary declarations but cannot reinterpret
those structural declarations. Formatters may place the constructor and
metadata near the beginning as a convention without making that order part of
the language.

Constructor arguments are immutable for the lifetime of an instance. They are
not application state. Mutable private state belongs to a
[task](tasks.md), while interaction remains governed by protocols, effects, and
capabilities.

## Context selection

`@` selects a path in the `Package` or `Module` context in which the function is
lexically defined:

```topal
process-request is fn ( request : Request ) -> Result ( Response, ContextErrorCode )
  (@ diagnostics) log "Processing request"
  handle-request request
```

The selected path may begin with a constructor argument, another declaration
introduced directly into that constructed context, or an artifact metadata
namespace available in the defining facade:

```topal
actual-version is fn () -> Version
  @ package version
```

The `package` namespace is not implicitly present in the function's lexical
scope, even when the function is declared in `package.t`; `package version`
without `@` is an error there. `@` never searches ordinary file scopes, a
parent directory, or a caller's context. An unknown `@` path is therefore a
local context error rather than the start of automatic namespace traversal.
Ordinary unqualified resolution within a function is limited to its parameters
and nested lexical scopes. Other namespaces require an explicit qualified path.

The defining context remains part of a function value when that value is
passed or stored. Closures retain only the contextual arguments they use.
Structured child tasks use the same immutable context instance as their parent.

## Construction with `use`

`use` supplies constructor arguments and makes the resulting namespace instance
available:

```topal
use logger (
  destination is stderr,
  minimum-level is Warning
)
```

The parenthesized construction value uses the ordinary comma notation for a
record. Its general contextual type is:

```topal
Record ( Identifier, Object )
```

Each label is an `Identifier` naming a constructor parameter and each
associated value is an `Object`. The selected `Package`, `Module`, or language
constructor refines that general shape to its declared parameter names and
classifications. Whitespace alone never separates construction arguments;
unknown, duplicate, and missing associations are errors.

Selection values such as a language or package version use the same
construction record as every other argument:

```topal
use lang topal (
  version is v1.5,
  features is ( testing )
)

use package org.example.rendering (
  version is v2.4,
  features is ( gpu, png )
)
```

There is no second positional `version` syntax. For language bootstrap, the
stable parser recognizes this same record notation before the selected language
assigns the fields their types and full meaning.

An instance may be given a local name when more than one configuration is
needed:

```topal
primary is use logger (
  destination is application-log
)

audit is use logger (
  destination is audit-log
)
```

Different argument values construct different instances. Topal does not merge
arguments across `use` declarations. A shared configuration is constructed
once and the resulting scope is bound or published explicitly.

Construction is checked before an instance is used. Every required argument
must be supplied exactly once unless it has a default, and the construction
dependency graph must be acyclic.

A language context may be constructed and named after the mandatory initial
bootstrap selection:

```topal
legacy is use lang topal (
  version is v1.0,
  features is ()
)
```

This construction is inert: it does not change how following source is parsed.
A later top-level `use legacy` activates the complete named language context
from that declaration forward. Construction and activation are static, and
activation is permitted only between complete top-level declarations.

The owner of an instance supplies its arguments:

- a package or module `use` constructs the selected dependency;
- a library consumer constructs the context underlying the selected library
  view; and
- deployment constructs the root `Package` underlying an application and must
  supply its required runtime arguments.

Library and application constructor requirements are therefore part of their
external interfaces even though `library.t` and `application.t` do not declare
separate constructors. Tests may construct the same context with alternative
arguments.

## Static construction and interface shape

An argument used to determine dependencies, types, declarations, visibility, or
other namespace structure must be statically known. An argument used only by
runtime function bodies may be an immutable runtime value:

```topal
Module (
  format : LogFormat,
  destination : LogDestination
)

if @ format = Json
  write-json is fn ( message : Message ) -> Unit
    (@ destination) send encode-json message

  pub write-json
```

Here `format` must be static because it controls the public interface.
`destination` may be supplied at runtime. The compiler infers the requirement
from use and reports it at the constructor argument. The top-level evaluation
of `package.t` remains static, so a `Package` argument used by that evaluation
must be static. A runtime `Package` argument may be selected only from inside a
runtime body.

Static constructor arguments which affect dependency discovery or compiled
interfaces form part of package resolution and reproducible build identity.
The package lock records them alongside the exact selected release.

## Features as ordinary arguments

Features have no privileged selection mechanism. `features` is a conventional
constructor argument interpreted by statically evaluated package, module, or
language code:

```topal
use package org.example.rendering (
  version is v7.2,
  features is ( gpu, png )
)
```

The selected implementation may use that value to enable dependencies,
declarations, and published members. Feature types define the permitted values.
No behavior follows merely from naming an argument `features`.

Feature sets do not accumulate or unify implicitly. Each `use` supplies the
complete constructor arguments for one instance. In particular, a later
language selection constructs a new language context:

```topal
use lang topal (
  version is v1.5,
  features is ( realtime )
)

production-declarations

use lang topal (
  version is v1.5,
  features is ( testing )
)

test-declarations
```

The second selection replaces the first context from its occurrence forward;
there is no separate clearing operation.

## Capabilities, endpoints, and containment

Constructor arguments may be immutable values such as protocol-specific
endpoints:

```topal
Module (
  user-service : Endpoint UserService
)

load-user is fn ( id : UserId ) -> Result ( User, ContextErrorCode )
  @ user-service request GetUser id
```

Selecting the endpoint is pure, total, and stable. The protocol call retains
its declared communication effects and fallibility. Constructor selection is
not a mechanism for hiding effects.

Contained diagnostic functions may likewise be supplied as capabilities.
Their inability to affect application state or control flow is a property of
their capability and effect contracts, not of `@`. Foreign implementations
which claim containment require a trusted adapter.

The compiler records the direct contextual selections of a function and
their transitive use through calls. Constructor-backed selections support
composition checking, documentation, specialization, and erasure of unused
context plumbing without adding ordinary source parameters to every function
declaration.

Static functions may select only statically known constructor arguments.

## Provisional grammar

The `@` symbol is a prefix scope selector rather than part of an identifier. Its
canonical formatting includes a space:

```topal
@ minimum-level
```

Conceptually:

```ebnf
context-selection   = "@" qualified-identifier ;
package-constructor = "Package" constructor-pattern ;
module-constructor  = "Module" constructor-pattern ;
use-construction    = "use" qualified-identifier
                      [ construction-record ] ;
construction-record = "("
                      [ construction-association
                        { "," construction-association } ]
                      ")" ;
construction-association = identifier "is" object ;
metadata-declaration = metadata-namespace qualified-identifier "is" object ;
metadata-namespace = "package" | "library" | "application" ;
source-provenance-declaration =
  ( "license" | "copyrights" ) "is" object ;
```

`context-selection` is a prefix expression and otherwise follows the normal
application and left-to-right grouping rules. The initial language construction
uses this same record syntax. Its stable bootstrap parser recognizes basic
static construction objects; the selected language assigns the fields their
meaning. Metadata declarations are permitted only at the top level of their
matching special file; use from a function requires `@` followed by the
qualified metadata path.

An unqualified `source-provenance-declaration` is permitted only in a source
file's root scope. Unlike qualified package or artifact metadata, it establishes
forward source state: each declaration replaces its own active value for
subsequent declarations in that file without changing the other provenance
field.
