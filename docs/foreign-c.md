# C access libraries

Topal uses the C ABI as its first and only foreign-language calling boundary.
“C ABI” is never sufficient identity by itself: an access library fixes the
target triple, platform ABI, object format, data model, compiler qualification,
calling convention, value layouts, symbols, effect contract, transfer rules,
and binary digest.
C++ and Rust native ABIs are deliberately outside this design. Code written in
another language remains usable when its producer publishes a C-compatible
interface.

## ABI language feature

A generated access library selects the `abi` language feature:

```topal
use language (
  version is v0.1,
  features is ( abi )
)
```

The source contains an `abi-library` record, one `abi-function-N` record per
function, and ordinary published Topal functions. These records describe the
schema, target, platform ABI, source and binary digests, linker symbol, calling
convention, ordered external layouts, unwind policy, and transfer policy. The
compiler compares this readable Topal model with the canonical machine
manifest before either representation is trusted.

The access function is an adapter, not exposure of the compiler's private
representation. It converts Topal values to their declared external layouts,
calls the C symbol using the qualified target C convention, validates the
result, and converts it back. No private Topal pointer, aggregate, allocator,
continuation, task, or callable crosses the boundary.

## First static subset

The first qualified profile is:

```text
target:          x86_64-unknown-linux-gnu
object format:   ELF64 little-endian
platform ABI:    System V AMD64
frontend:        Clang 22
linkage:         static archive selected explicitly by one access library
C declarations:  non-variadic functions carrying Clang's const contract
values:          C int parameters; C int or void result
transfer:        copied value
unwind:          forbidden across the adapter
```

Topal `Int` remains arbitrary precision. The adapter checks representability
before forming a C `int`; failure terminates at the boundary with the stable
`E-C-ABI-RANGE` diagnostic rather than truncating, wrapping, or exposing a
private layout. A C `int` result is sign-extended and constructs a new Topal
`Int`.

`topal-c-bindgen` parses the header with the target-qualified Clang frontend,
rejects every unsupported declaration, verifies the archive architecture and
exported symbols with LLVM tools, and atomically emits the access-library
directory. It records SHA-256 digests for the header and archive. `topalc`
rechecks those digests before compilation and records them as native artifact
dependencies.

The required Clang `const` function attribute is the first profile's explicit
effect contract. With only integer inputs it promises that the function's
observable result depends on its arguments and that the call has no observable
side effects. The linked implementation is trusted to honor that declaration;
Clang and Topal cannot prove it from machine code. Stateful, I/O-performing,
nondeterministic, or otherwise effectful C functions are rejected until a
Topal effect identity and capability contract can describe them.

## Deliberate limitations

The first profile rejects pointers, arrays, structures, unions, enumerations,
callbacks, variadics, macros, inline-only functions, global variables,
thread-local state, `longjmp`, implementation-specific calling conventions,
effectful functions, and every ownership or lifetime rule that would need to
be guessed. Later C
profiles may add a type only together with explicit layout, validity,
ownership, aliasing, failure, effect, and callback rules.

Static archives may themselves contain only dependencies resolvable within the
freestanding final link. The first profile adds no default C runtime, startup
object, dynamic interpreter, or implicit library. Shared objects are a
separate increment because their loader, search, identity, and deployment
semantics are observable.
