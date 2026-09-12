# Compiler conformance roadmap

This roadmap turns the approved compiler direction into reviewable increments.
Each increment ends in a PR and extends the same compiler rather than creating
disposable prototypes. A domain is complete only when every applicable stable
rule has an explicit compiler disposition and shared executable regression
evidence.

| Increment | Language and tool closure | Status |
| ---: | --- | --- |
| 1 | LLVM 22 pipeline, Linux x86-64 freestanding startup/syscalls, O0, DWARF/GDB, native metadata, Unit/Boolean/bounded exact Int/positional products, immutable bindings, eager Boolean and checked integer operations, ordinary nonrecursive function specialization, Boolean decisions | complete |
| 2a | arbitrary finite `Int` representation, literals, negation, absolute value, addition, subtraction, multiplication, equality, ordering, decimal output, GDB rendering, and freestanding allocation | complete |
| 2b | finite `Int` identities/division/modulo/power, finite exact `Rational`, static zero-divisor diagnostics, exact equality/ordering, and direct three-way comparison | complete |
| 2c-a | ordered comparison matchers and exhaustive decisions over `Comparison` values | complete |
| 2c-b1 | dynamic arithmetic `Result`/Error representation, construction, division, power, modulo, quotient/modulo, propagation, output, and debugging | complete |
| 2c-b2 | exact Rational-to-Int and Nat narrowing/validation plus contextual success projection | complete |
| 2c-b3 | structured Error observation and exhaustive Result/error-code decisions | complete |
| 2c-c | normatively completed infinity construction and arithmetic | planned |
| 2d-a | explicitly bounded finite numeric range construction, classification, membership, intersection, emptiness, and bound observation | complete |
| 2d-b | unbounded range construction and infinity endpoints after their prerequisite normative and runtime work | planned |
| 3 | complete function forms, recursion/totality evidence, overloads, records, enums, unions, constraints, capabilities, and decisions | planned |
| 4 | strings, Unicode operations, fundamental containers, traversal, and representation-safe allocation | planned |
| 5 | generators, suspension, closure environments, linear close/resume behavior | planned |
| 6 | module/package/application construction, source and compiled libraries, GEIR instantiation, incremental and link-time compilation | planned |
| 7 | effects, resources, layouts, locations, tasks, deterministic scheduling, transactions, time, static flow, and platform packages | planned |
| 8 | native serialization, introspection, contracts/evidence, implementation plans, information flow, and remaining `v0.2` assurance behavior | planned |
| 9 | complete cross-tool rule audit, optimized-level admission, LTO/sanitizer/coverage/PGO dispositions, and whole-core parity qualification | planned |

## Increment acceptance

Every increment shall:

1. cite all newly accepted specification rule IDs in compiler requirements and
   functional tests;
2. compile and run the same `.t` sources used by the interpreter, without a
   copied compiler version;
3. compare interpreter and `-O0` executable observations for the supported
   shared corpus;
4. reject still-unsupported constructs with stable compiler diagnostics;
5. retain distinct compiler-build, compiler-execution, and interpreter resource
   measurements;
6. verify LLVM IR, native artifact metadata, binary dependency freedom, and
   applicable debug information; and
7. update this table, `se/traceability.md`, and the compiler requirement file.

Increment 1's bounded integer lowering accepted an operation only when static
range evidence proved that its exact result fit the initial signed 64-bit
representation. Increment 2a removes that boundary with a freestanding Topal
numeric runtime and a private, dynamically sized representation. Increment 2b
adds finite exact division and Rational values. Increment 2c-a adds the fully
normative Comparison decision forms; 2c-b1 adds the initial typed arithmetic
Result ABI and failure paths, while 2c-b2 and 2c-b3 retain narrowing,
projection, observation, and Result decisions. Increment 2c-c retains the
infinity work that requires normative completion. Increment
2d-a adds the fully normative explicitly bounded finite range subset; 2d-b
closes unbounded and infinite endpoints after increment 2c-c.
Range-based collection selection remains grouped with containers. The roadmap
then proceeds to general control flow and user-defined value representations.
