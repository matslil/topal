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
ordinary algorithm:

```topal
# logger/module.t

Module (
  destination : LogDestination
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
  features : Set PackageFeature default ()
  diagnostics : Diagnostics
)

package is se.example.calculator
version is v5.3.1
```

The capitalized structural words distinguish context construction from `fn`
declarations and ordinary value bindings. The exact ordering of the constructor
declaration and mandatory package metadata remains provisional.

Constructor arguments are immutable for the lifetime of an instance. They are
not application state. Mutable private state belongs to a
[task](tasks.md), while interaction remains governed by protocols, effects, and
capabilities.

## Context selection

`@` selects a member of the `Package` or `Module` context in which the function
is lexically defined:

```topal
process-request is fn ( request : Request ) -> Result Response
  (@ diagnostics) log "Processing request"
  handle-request request
```

The selected member may be a constructor argument or another declaration
introduced directly into that constructed context. `@` never searches ordinary
file scopes, a parent directory, a caller's context, or a library or application
facade. An unknown `@` name is therefore a local context error rather than the
start of automatic namespace traversal. Ordinary unqualified resolution within
a function is limited to its parameters and nested lexical scopes. Other
namespaces require an explicit qualified path.

The defining context remains part of an algorithm value when that value is
passed or stored. Closures retain only the contextual arguments they use.
Structured child tasks use the same immutable context instance as their parent.

## Construction with `use`

`use` supplies constructor arguments and makes the resulting namespace instance
available:

```topal
use logger (
  destination is stderr
  minimum-level is Warning
)
```

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
runtime algorithm bodies may be an immutable runtime value:

```topal
Module (
  format : LogFormat
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
use package org.example.rendering version v7.2 (
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
use lang topal v1.5 (
  features is ( realtime )
)

production-declarations

use lang topal v1.5 (
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

load-user is fn ( id : UserId ) -> Result User
  @ user-service request GetUser id
```

Selecting the endpoint is pure, total, and stable. The protocol call retains
its declared communication effects and fallibility. Constructor selection is
not a mechanism for hiding effects.

Contained diagnostic algorithms may likewise be supplied as capabilities.
Their inability to affect application state or control flow is a property of
their capability and effect contracts, not of `@`. Foreign implementations
which claim containment require a trusted adapter.

The compiler records the direct contextual selections of an algorithm and
their transitive use through calls. Constructor-backed selections support
composition checking, documentation, specialization, and erasure of unused
context plumbing without adding ordinary source parameters to every algorithm
declaration.

Static algorithms may select only statically known constructor arguments.

## Provisional grammar

The `@` symbol is a prefix scope selector rather than part of an identifier. Its
canonical formatting includes a space:

```topal
@ minimum-level
```

Conceptually:

```ebnf
context-selection   = "@" identifier ;
package-constructor = "Package" constructor-pattern ;
module-constructor  = "Module" constructor-pattern ;
use-construction    = "use" qualified-identifier
                      [ "version" version ]
                      [ argument-map ] ;
```

`context-selection` is a prefix expression and otherwise follows the normal
application and left-to-right grouping rules. The initial language construction
retains a small stable bootstrap grammar for its version and optional basic
static argument map; the selected language assigns those arguments their
meaning.
