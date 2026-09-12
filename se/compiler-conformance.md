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
| 2c-b4 | qualified arithmetic ErrorCode values, equality, function passage, display, and debugging | complete |
| 2c-c | normatively completed infinity construction and arithmetic | planned |
| 2d-a | explicitly bounded finite numeric range construction, classification, membership, intersection, emptiness, and bound observation | complete |
| 2d-b | unbounded range construction and infinity endpoints after their prerequisite normative and runtime work | planned |
| 2e | Nat constraint-evidence forgetting for exact equality, ordering, three-way comparison, mixed Nat/Int/Rational comparison, and derived product equality | complete |
| 3a | source-ordered statically decidable scalar overloads, complete-header forward calls, and basic static nullary, unary, and binary functions | complete |
| 3b1 | root-scope payload-free nominal enum declarations, classification, equality, functions, display, exhaustive decisions, and debugging | complete |
| 3b2-a | direct explicit early return from admitted linear function bodies | complete |
| 3b2-b1 | value-producing lexical blocks with fresh binding scope, shadowing, non-escape, and lexical DWARF | complete |
| 3b2-b2 | distinct zero-data Completed evidence, equality, scalar function passage, retained call dependency, display, and debugging | complete |
| 3b2-b3 | ordinary positional-product prefix calls and typed discard input patterns without source/debug bindings | complete |
| 3b2-b4 | `Optional Int`/`Optional String` construction, contextual absence, scalar function passage, exhaustive decisions, display, `Optional Int` equality, and debugging | complete |
| 3b2-b4a | extend the same native Optional paths to exact Rational payloads and recursively derived product equality | complete |
| 3b2-b5a | same-classifier positional-product equality recursively derived from admitted field equality | complete |
| 3b2-b5b | anonymous labeled-record construction, static field selection, canonical display, and projected scalar debugging | complete |
| 3b2-b5c | recursive lexicographic Tuple ordering, label-aligned Record equality, per-field exact numeric conversion, and scalar-result debugging | complete |
| 3b2-b5d | immutable Record reconstruction with exact field classification, decomposed lowering, original-value preservation, and projected scalar debugging | complete |
| 3b2-b5e1 | proven direct decreasing unary Int recursion, multiple proven self-calls, conservative recursive parameter facts, and recursive-frame debugging | complete |
| 3b2-b5e2 | proven direct increasing unary Int recursion and positive multi-unit progress in both directions | complete |
| 3b2-b5e3 | range-preserving direct decreasing and increasing unary Nat recursion with proof-backed constraint evidence | complete |
| 3b2-b5e4 | explicit single-parameter Int/Nat `Decreases` evidence over multi-parameter scalar recursion | complete |
| 3b2-b5e5 | remaining function forms, return-through-block cleanup, remaining recursion/totality evidence, nested declarations, record storage/ABI, unions, constraints, capabilities, and decisions | planned |
| 4a | prospective UTF-8 String byte count through the native descriptor and arbitrary-precision Int runtime | complete |
| 4b1 | exact preserved-sequence String equality and derived `Optional String` equality | complete |
| 4b2 | empty String construction, adjacent literal composition, exact concatenation, emptiness, dynamic canonical display, and GDB inspection | complete |
| 4b3a | closed Character constraint validation, retained function/equality evidence, lossless String forgetting, and debugging | complete |
| 4b3b | closed Character/entry counting and exact indexing with Optional Character passage, decisions, display, and debugging | complete |
| 4b3c | closed pinned-Unicode uppercase, lowercase, full case-fold, NFC/NFD normalization, canonical equivalence, and debugging | complete |
| 4b3d | dynamic Character and Unicode operations, remaining strings, fundamental containers, traversal, and representation-safe allocation | planned |
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
projection, observation, and Result decisions. Increment 2c-b4 exposes the same
closed nominal arithmetic-code vocabulary as direct qualified values. Increment
2c-c retains the infinity work that requires normative completion. Increment
2d-a adds the fully normative explicitly bounded finite range subset; 2d-b
closes unbounded and infinite endpoints after increment 2c-c.
Range-based collection selection remains grouped with containers. Increment 2e
reuses the validated Nat value as its exact Int representation for comparison,
without an unsigned conversion or a second numeric runtime. Increment 3a
admits the statically decidable scalar overload, complete-header acyclic forward
calls, and basic static-function foundation. Increment 3b1 adds the first
root-scope user-declared nominal value
representation without conflating payload-free `Enum` with general `Union`;
3b2-a makes direct early return a mandatory frontend control-flow boundary;
3b2-b1 adds ordinary lexical block values and truthful nested debug scope;
3b2-b2 adds the distinct zero-data Completed value without erasing its call
dependency into Unit; 3b2-b3 completes positional-product prefix application
and typed discard inputs without inventing bindings; 3b2-b4 adds a distinct
freestanding Optional representation; 3b2-b4a admits Rational payloads without
changing that representation; 3b2-b5a derives same-classifier
positional-product equality without introducing an aggregate ABI; 3b2-b5b adds
decomposed anonymous records, exact static selection, and source-ordered display
without inventing aggregate storage or debug layout; 3b2-b5c adds recursive
lexicographic Tuple ordering and label-aligned Record equality with per-field
exact numeric conversion; 3b2-b5d adds source-ordered immutable Record
reconstruction without introducing aggregate storage; 3b2-b5e1 adds the
directly proven decreasing `Int` closure; 3b2-b5e2 adds its increasing dual and
multi-unit positive steps in both directions; 3b2-b5e3 adds range-preserving
direct `Nat` recursion without a distinct numeric representation; 3b2-b5e4
extends the shared single-parameter `Decreases` proof across a larger scalar
state; and 3b2-b5e5 continues with nested declarations, remaining recursive and
nested-block exit control flow, cleanup-bearing scopes, and the remaining
user-defined value representations. Increment 4a admits the encoding-observation
byte count without
attaching an encoding or importing a foreign String runtime; 4b1 adds exact
preserved-sequence equality without normalization or locale policy; and 4b2
adds empty construction, literal composition, exact dynamic concatenation,
emptiness, and canonical dynamic display. Increment 4b3a retains statically
proved Character evidence over the same descriptor; 4b3b adds closed
observations under pinned segmentation; 4b3c adds closed pinned-Unicode
transformations and canonical equivalence; 4b3d retains dynamic observations
plus the remaining Unicode and container work.
