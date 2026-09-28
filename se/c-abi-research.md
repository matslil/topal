# C ABI research and Topal disposition

The C language standard defines source behavior, not one portable binary ABI.
The first Topal profile is consequently qualified by the complete
`x86_64-unknown-linux-gnu` target and the System V AMD64 processor supplement.
That ABI classifies arguments and results, defines register and stack use, and
depends on the ELF object and dynamic-linking model. LLVM's `ccc` denotes the
selected target C convention; LLVM `fastcc` explicitly does not conform to an
external ABI and remains appropriate only for Topal-private calls.

Primary references checked for this decision:

- [System V AMD64 ABI](https://gitlab.com/x86-psABIs/x86-64-ABI);
- [ELF object format and dynamic linking](https://gabi.xinuos.com/elf/);
- [LLVM calling conventions and data layout](https://llvm.org/docs/LangRef.html);
- [Clang tooling interfaces](https://clang.llvm.org/docs/Tooling.html); and
- [Clang AST matchers](https://clang.llvm.org/docs/LibASTMatchers.html).

## Difference from Topal

Topal values are semantic and may use compiler-private representations. In the
current native ABI, `Int` is an immutable arbitrary-precision object passed as
a private pointer, while a C `int` in the qualified profile is a signed 32-bit
value passed according to the System V AMD64 rules. Topal internal functions
use versioned private `fastcc` signatures. Therefore neither the Topal value
pointer nor the private function signature can be linked directly to C.

The adapter pattern isolates this difference:

```text
Topal call -> private fastcc wrapper -> checked value conversion
           -> target ccc symbol      -> checked Topal construction
```

Headers provide declaration information but not binary presence. Archives
provide symbols and machine code but not sufficient semantic types. A valid
access library therefore binds a Clang-checked header and an LLVM-inspected
archive by digest. Compilation flags and frontend version are part of
provenance because preprocessing and implementation-defined C choices can
change the interface.

## Model completeness

The general C ABI model has fields for target and toolchain identity, object
kind, symbols, calling convention, ordered parameter and result layouts,
variadic status, transfer and ownership, nullability and aliasing, destruction,
unwind and nonlocal control, callbacks and thread entry, effects, dependencies,
and input provenance. The version-one schema represents all these dimensions
but admits only the closed copied `int`/`void` subset whose unsupported
dimensions have one unambiguous value. Its sole effect value is
`no-observable-effect`, admitted only when Clang reports the function's `const`
attribute. That declaration is a producer contract rather than a binary proof.
Expanding the schema precedes expanding the admitted declarations.

This choice deliberately avoids native C++ and Rust ABI modeling. Producers in
those languages can supply a C wrapper, keeping Topal's trusted boundary small
and stable without claiming that C makes the wrapped implementation safe.
