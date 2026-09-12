# Native compiler requirements

These requirements refine `TOPAL-REQ-COMPILER-001`,
`TOPAL-REQ-NATIVE-PLATFORM-001`, `TOPAL-REQ-NATIVE-ABI-001`,
`TOPAL-REQ-NATIVE-ARTIFACT-001`, `TOPAL-REQ-NATIVE-DEBUG-001`, and
`TOPAL-REQ-LLVM-001` for the admitted `topalc` increments.

## TOPAL-COMP-TARGET-001 — Linux x86-64 qualification

The compiler shall accept only `x86_64-unknown-linux-gnu`, emit the exact
qualified data layout and x86-64 CPU baseline, use position-independent code,
and reject every other target before creating an output.

## TOPAL-COMP-LLVM-001 — Verified LLVM 22 pipeline

The compiler shall locate an explicit or toolchain-provided LLVM 22 suite,
reject another major, assemble and verify every emitted module, use `llc` for
O0 object generation with frame pointers, and use LLD for executable linking.
Tool failure shall preserve the responsible command's diagnostics and shall not
publish a partial requested output.

## TOPAL-COMP-PLATFORM-001 — No foreign runtime dependency

An executable shall define `_start`, perform complete standard-output writes
and termination through the Linux x86-64 syscall ABI, and link as a static PIE
without startup files, default libraries, a dynamic interpreter, `DT_NEEDED`
entries, or undefined symbols.

## TOPAL-COMP-O0-001 — Exact first-slice lowering

The first compiler increment shall lower Unit, Boolean, signed-64-bit-
representable exact Int values, static strings, positional products, immutable
bindings, explicit discard, eager Boolean operations, integer addition,
subtraction, multiplication, equality and ordering, ordinary nonrecursive
scalar functions, and complete Boolean decisions. It shall accept integer
operations only when shared static range evidence proves their exact result is
representable and shall reject every other construct with
`E-COMPILER-UNSUPPORTED`.

This requirement covers `TOPAL-SYN-CONTEXT-001`, `TOPAL-SYN-BIND-001`,
`TOPAL-SYN-STRING-001`,
`TOPAL-TYPE-PRODUCT-001`, `TOPAL-TYPE-BOOLEAN-001`,
`TOPAL-TYPE-EQUALITY-001`, `TOPAL-NUM-INT-001`, `TOPAL-NUM-ADD-001`,
`TOPAL-NUM-NEG-001`, `TOPAL-NUM-SUB-001`, `TOPAL-NUM-MUL-001`,
`TOPAL-NUM-COMPARE-001`, `TOPAL-FUNCTION-ORDINARY-001`, and
`TOPAL-DECISION-BOOLEAN-001` within the stated incremental boundary.

## TOPAL-COMP-INT-001 — Arbitrary finite Int runtime

The compiler shall remove the increment-1 signed-64-bit admission boundary and
represent every admitted finite `Int` with dynamically sized exact storage. At
O0, literals, negation, absolute value, addition, subtraction, multiplication,
equality, ordering, function passage, decision joins, decimal output, and GDB
inspection shall preserve the normative value for operands of any size that
available address space can hold.

The private `topal-native/4` representation shall be canonical, immutable, and
hidden from foreign calling conventions. Its allocator and output routines
shall use only the qualified Linux x86-64 system-call boundary, detect mapping
failure, introduce no C/C++ runtime dependency or undefined helper symbol, and
retain allocated values safely until process termination. Reclamation beyond
that process-lifetime policy is deferred until reachability-bearing values are
admitted and shall not change source observations.

This requirement covers `TOPAL-NUM-INT-001`, `TOPAL-NUM-NEG-001`,
`TOPAL-NUM-ABS-001`, `TOPAL-NUM-ADD-001`, `TOPAL-NUM-SUB-001`,
`TOPAL-NUM-MUL-001`, `TOPAL-NUM-COMPARE-001`, and the applicable Int cases of
`TOPAL-TYPE-EQUALITY-001`.

It realizes `TOPAL-COMPILER-INT-001` for compiler increment 2a.

## TOPAL-COMP-EXACT-001 — Finite exact-number runtime

The compiler shall represent finite Rational values as immutable pairs of
canonical arbitrary-precision Int numerator and positive denominator objects.
It shall implement relocation-free literals and identities, closed and
specialization-proven construction, canonical Int embedding, exact Int and
Rational division, Rational negation, absolute value, addition, subtraction,
multiplication and powers, Int power, Euclidean modulo and quotient/modulo,
same-domain and mixed exact equality/ordering, direct three-way Comparison
values, textual output, function passage, and GDB inspection.

Normalization shall use exact greatest-common-divisor and division algorithms.
The implementation shall use no fixed source-value width, C/C++ runtime,
foreign allocator, arithmetic helper, load-time pointer relocation, or dynamic
loader. The compiler shall diagnose statically evident zero divisors and fail
closed on dynamic arithmetic-error paths until typed Result lowering is
implemented; generated code shall not terminate in place of an admitted Result.

This requirement covers `TOPAL-NUM-RATIONAL-001`,
`TOPAL-NUM-RATIONAL-CONSTRUCT-001`, `TOPAL-NUM-RATIONAL-LITERAL-001`,
`TOPAL-NUM-RAT-NEG-001`, the finite Rational case of `TOPAL-NUM-ABS-001`,
the admitted Int and Rational cases of `TOPAL-NUM-ZERO-001` and
`TOPAL-NUM-ONE-001`, `TOPAL-NUM-RAT-ADD-001`, `TOPAL-NUM-RAT-SUB-001`,
`TOPAL-NUM-RAT-MUL-001`, `TOPAL-NUM-RAT-DIV-001`,
`TOPAL-NUM-INT-RATIONAL-CONVERT-001`, `TOPAL-NUM-DIV-001`,
`TOPAL-NUM-DIVZERO-001`, `TOPAL-NUM-INT-MODULO-001`,
`TOPAL-NUM-INT-QUOTIENT-MODULO-001`, `TOPAL-NUM-POW-001`,
`TOPAL-NUM-RAT-POW-001`, `TOPAL-NUM-RAT-NEG-POW-001`,
`TOPAL-NUM-COMPARE-001`, `TOPAL-NUM-THREE-WAY-COMPARE-001`, and applicable
exact-number cases of `TOPAL-TYPE-EQUALITY-001` and
`TOPAL-TYPE-ORDERING-001`.

It realizes `TOPAL-COMPILER-EXACT-001` for compiler increment 2b.

## TOPAL-COMP-RANGE-001 — Finite exact ranges

The compiler shall represent explicitly bounded finite `Range Int` and
`Range Rational` values as immutable opaque handles retaining exact lower and
upper endpoint objects and canonical Boolean inclusivity states. It shall
implement all four range constructors, canonical mixed endpoint conversion,
classification, ordinary function passage and decision joins, both membership
operand orders, same-domain intersection, emptiness, bound and inclusivity
observation, source-form output, and GDB inspection.

Range lowering shall not enumerate members, normalize open endpoints by
arithmetic, expose the private header to a foreign calling convention, require
a C/C++ runtime, or introduce load-time pointer relocations. Unbounded forms,
infinite endpoints, and collection selection remain outside this increment.

This requirement covers `TOPAL-RANGE-BOUNDS-001`,
`TOPAL-RANGE-MEMBERSHIP-001`, `TOPAL-RANGE-RATIONAL-001`,
`TOPAL-RANGE-CLASSIFIER-001`, `TOPAL-RANGE-INTERSECTION-001`,
`TOPAL-RANGE-EMPTY-001`, and `TOPAL-RANGE-BOUND-001`. It realizes
`TOPAL-COMPILER-RANGE-001` for compiler increment 2d-a.

## TOPAL-COMP-DEBUG-001 — DWARF and GDB

Debug-enabled O0 output shall map generated source functions, parameters,
immutable scalar locals, and instructions to Topal files and source locations,
emit DWARF 5 through LLVM, retain frame pointers, provide GDB renderers for
private arbitrary-precision Int, Rational, `Range Int`, and `Range Rational`
objects, and pass automated GDB breakpoint, value, and backtrace scenarios.

## TOPAL-COMP-ARTIFACT-001 — Canonical sidecar metadata

Every requested output shall receive a canonical
`topal.native-artifact/1` JSON sidecar containing the complete admitted
target, ABI, compiler, LLVM, optimization, debug, source, interface, dependency,
export, platform, evidence, and provenance fields. The metadata API shall reject
schema, target, revision, ordering, duplicate-identity, and digest errors before
an artifact is consumed.

## TOPAL-COMP-TEST-001 — Shared native regressions

Compiler tests shall compile and execute the unchanged interpreter regression
sources listed by `topalc test --list`, compare output to the interpreter, and
verify freestanding ELF and GDB properties. Resource tooling shall measure
compiler builds and executable runs under identities separate from the
interpreter baseline.
