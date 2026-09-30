# C ABI access conformance

## TOPAL-C-ABI-CONTEXT-001 — Explicit ABI feature

A C access-library source SHALL select language revision `v0.1` with feature
`abi`. Ordinary application source SHALL select the library explicitly. The
feature SHALL grant vocabulary only to the generated access-library context.

## TOPAL-C-ABI-IDENTITY-001 — Complete target identity

An access library SHALL identify its schema, target triple, platform ABI,
object format implied by that qualified profile, Clang version, header digest,
binary kind, binary digest, and library version. A consumer SHALL reject a
missing, unknown, contradictory, or noncanonical identity before code
generation.

## TOPAL-C-ABI-MODEL-001 — Self-describing Topal representation

The Topal `abi` source SHALL be the sole canonical description of the access
library. Its library record SHALL contain the schema, library identity and
version, target, platform ABI, object format, binary kind, Clang identity, and
the relative filename and digest of each input artifact. Each function record
SHALL contain its Topal name, linker symbol, calling convention, variadic
status, ordered parameter names and layouts, result layout, effect, unwind
rule, and transfer rule. A compiler SHALL reject a noncanonical source,
missing or duplicate field, unsafe artifact path, or disagreement between a
function record and its published Topal declaration. A producer and consumer
SHALL NOT require a parallel non-Topal manifest.

## TOPAL-C-ABI-STATIC-001 — Qualified static archive

For schema `topal-c-abi/1`, the target SHALL be
`x86_64-unknown-linux-gnu`, the platform ABI SHALL be `sysv-amd64`, and the
binary SHALL be an ELF x86-64 static archive. Every declared external symbol
SHALL be a defined external archive symbol. The final static PIE SHALL contain
no undefined symbol, dynamic interpreter, `DT_NEEDED` entry, implicit startup
object, or default library.

## TOPAL-C-ABI-FUNCTION-001 — Initial function subset

The initial function subset SHALL admit only non-variadic ordinary C functions
that carry Clang's `const` function attribute, whose parameters are C `int`,
and whose result is C `int` or `void`. The canonical Topal model SHALL record
`no-observable-effect`. The producer SHALL ensure that the linked
definition honors the declared contract; functions with observable state,
I/O, nondeterminism, or other effects are outside this profile. Every
parameter is a copied Topal `Int`; every `int` result constructs a new Topal
`Int`; `void` maps to `Unit`. The adapter SHALL use target C calling convention
`ccc` externally and SHALL preserve the private Topal calling convention only
inside the executable.

## TOPAL-C-ABI-RANGE-001 — No numeric truncation

Before a Topal `Int` is passed as C `int`, the adapter SHALL validate the
inclusive range `-2147483648 ..= 2147483647`. An invalid value SHALL produce
`E-C-ABI-RANGE` and terminate the boundary; it SHALL NOT truncate, wrap, call
the C function, or expose a private representation. Every C `int` result SHALL
be sign-extended exactly.

## TOPAL-C-ABI-REJECT-001 — Fail-closed translation

The translator SHALL reject a variadic declaration, unsupported type,
conflicting declaration, absent symbol, wrong target, modified input, invalid
identifier, or ambiguous ownership rule. It SHALL NOT silently omit a header
function or infer pointer ownership, length, lifetime, nullability, aliasing,
callback, or error semantics.

## TOPAL-C-ABI-PROVENANCE-001 — Reproducible inputs

Generation SHALL record Clang identity and SHA-256 digests of the header and
selected archive or shared object. Compilation SHALL revalidate both digests
and SHALL record them in the native artifact dependency set and build identity.

## TOPAL-C-ABI-SHARED-001 — Closed shared-object deployment

For schema `topal-c-abi-shared/1`, the binary SHALL be an ELF x86-64 shared
object with an explicit SONAME identical to its access-library filename. It
SHALL contain no `DT_NEEDED` dependency, unresolved external symbol, load
initializer, or unload initializer. The linked executable SHALL name each and
only the selected SONAMEs through `DT_NEEDED`, use ELF interpreter
`/lib64/ld-linux-x86-64.so.2`, and use `$ORIGIN` lookup. The compiler SHALL
deploy the exact digested object beside the executable and SHALL reject a
different existing object with the same SONAME. Artifact metadata SHALL record
the object digest, SONAME requirement, and interpreter requirement.
