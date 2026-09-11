# Native compiler requirements

These requirements refine `TOPAL-REQ-COMPILER-001`,
`TOPAL-REQ-NATIVE-PLATFORM-001`, `TOPAL-REQ-NATIVE-ABI-001`,
`TOPAL-REQ-NATIVE-ARTIFACT-001`, `TOPAL-REQ-NATIVE-DEBUG-001`, and
`TOPAL-REQ-LLVM-001` for `topalc` increment 1.

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

## TOPAL-COMP-DEBUG-001 — DWARF and GDB

Debug-enabled O0 output shall map generated source functions, parameters,
immutable scalar locals, and instructions to Topal files and source locations,
emit DWARF 5 through LLVM, retain frame pointers, and pass automated GDB
breakpoint, value, and backtrace scenarios.

## TOPAL-COMP-ARTIFACT-001 — Canonical sidecar metadata

Every requested output shall receive a canonical
`topal.native-artifact/1` JSON sidecar containing the complete increment-1
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
