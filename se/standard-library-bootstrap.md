# Standard-library bootstrap contract

## Supported core

The toolchain's standard-library development baseline is the immutable Topal
language context identified by language version `v0.1` (`design-0`). Every
source file selects its own language version through the stable bootstrap
header. When a command-line version is absent, each tool selects its highest
supported revision; a command-line version remains only the interactive-input
default and never overrides a source file's selection.

This is a supported core revision for repository development and
interoperability testing, not a promise that the experimental project has made
a stable public release. A later language revision does not alter `v0.1`.

## Library boundary

The standard library is ordinary versioned Topal source built on the completed
core-language ledger. It may publish modules, types, functions, generators,
interfaces, capabilities, effects, protocols, layouts, and derived static
artifacts using the same visibility and type rules as application source.

The library receives no implicit reflection, authority, ambient resource,
compiler intrinsic, hidden syntax, unchecked memory operation, or privileged
namespace. Platform facilities enter through explicit published objects,
capabilities, effects, protocols, and checked layouts. A proposed primitive
which cannot be expressed through this boundary is a core-language design
change and follows the repository's human-decision procedure before library
code depends on it.

## Bootstrap order

1. The stable source bootstrap selects the file's immutable language context.
2. The shared frontend parses and validates the module without executing it.
3. Package and module loading establishes only published, version-compatible
   dependencies.
4. Static construction and introspection derive typed ordinary artifacts
   without retaining automatic runtime reflection.
5. Runtime execution receives explicit arguments, capabilities, and effects;
   resource acquisition remains an ordinary explicit effect.

Library packages shall state their supported language revision and test their
public interfaces through the interpreter, scripted debugger, language server,
serialization codec, and later compiler wherever those tools apply. Native
serialization and GEIR artifacts retain exact language revisions and are not a
cross-version escape hatch.

The implementation phases, shared interpreter/compiler architecture, and
stacked review procedure are defined in
[`standard-library-plan.md`](standard-library-plan.md).
The follow-on fundamental API completion series is defined in
[`fundamental-standard-library-completion.md`](fundamental-standard-library-completion.md).
That completed API is published directly through the flat `std` namespace.
Non-fundamental algorithm packages use separate namespaces so their growth does
not change fundamental names.

## Initial development gate

Standard-library work may begin when the complete workspace validation passes
and `se/core-language-coverage.md` contains no planned row. Each library
increment adds runnable commented examples and updates all affected source
tools, traces, specifications, and conformance evidence under the same change
procedure used for core-language increments.
