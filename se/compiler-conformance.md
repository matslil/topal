# Compiler conformance roadmap

This roadmap turns the approved compiler direction into reviewable increments.
Each increment ends in a PR and extends the same compiler rather than creating
disposable prototypes. A domain is complete only when every applicable stable
rule has an explicit compiler disposition and shared executable regression
evidence.

| Increment | Language and tool closure | Status |
| ---: | --- | --- |
| 1 | LLVM 22 pipeline, Linux x86-64 freestanding startup/syscalls, O0, DWARF/GDB, native metadata, Unit/Boolean/bounded exact Int/positional products, immutable bindings, eager Boolean and checked integer operations, ordinary nonrecursive function specialization, Boolean decisions | complete |
| 1a | shared validation and runtime erasure of legacy warning and structured diagnostic-identity controls | complete |
| 2a | arbitrary finite `Int` representation, literals, negation, absolute value, addition, subtraction, multiplication, equality, ordering, decimal output, GDB rendering, and freestanding allocation | complete |
| 2b | finite `Int` identities/division/modulo/power, finite exact `Rational`, static zero-divisor diagnostics, exact equality/ordering, and direct three-way comparison | complete |
| 2c-a | ordered comparison matchers and exhaustive decisions over `Comparison` values | complete |
| 2c-b1 | dynamic arithmetic `Result`/Error representation, construction, division, power, modulo, quotient/modulo, propagation, output, and debugging | complete |
| 2c-b2 | exact Rational-to-Int and Nat narrowing/validation plus contextual success projection | complete |
| 2c-b3 | structured Error observation and exhaustive Result/error-code decisions | complete |
| 2c-b4 | qualified arithmetic ErrorCode values, equality, function passage, display, and debugging | complete |
| 2c-b5 | `detail`, `cause`, and `source` Error observations with precise Optional payloads, canonical SourceLocation values, private passage, display, and debugging | complete |
| 2c-c | normatively completed infinity construction and arithmetic | planned |
| 2d-a | explicitly bounded finite numeric range construction, classification, membership, intersection, emptiness, and bound observation | complete |
| 2d-b | unbounded range construction and infinity endpoints after their prerequisite normative and runtime work | planned |
| 2e | Nat constraint-evidence forgetting for exact equality, ordering, three-way comparison, mixed Nat/Int/Rational comparison, and derived product equality | complete |
| 2f | root-scope nominal `ModNat`/`ModInt` declarations with direct finite ranges, proven checked construction, explicit reduction, wrapping arithmetic/negation, equality, ordering, comparison, private function passage, canonical display, and debugging | complete |
| 2f1 | earlier named finite-range operands and dynamic checked modular construction with source-located Result/Error composition and nominal Result debugging | complete |
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
| 3b2-b5e5 | overload-specific recursion identity and same-named acyclic cross-overload calls | complete |
| 3b2-b5e6 | complete decreasing and increasing mutual Int cycles with multiple proven next-member calls | complete |
| 3b2-b5e7 | range-preserving decreasing and increasing mutual Nat cycles with proof-backed constraint evidence | complete |
| 3b2-b5f | recursively composed private Tuple function results over admitted scalar leaves, target-derived DWARF layout, and GDB-visible aggregate bindings | complete |
| 3b2-b5g | field-wise Tuple results for Boolean, ordered-comparison, Comparison, Enum, Optional, and Result decisions | complete |
| 3b2-b5h | recursively composed private Tuple parameters, candidate-specific product-call normalization, exact aggregate prototypes, and GDB-visible parameters | complete |
| 3b2-b5i | order-preserving private Record parameters, results, nesting, decisions, exact aggregate prototypes, and GDB-visible named fields | complete |
| 3b2-b5j | retained named Function values, typed aliases, binding chains, captured overload sets, direct specialization calls, canonical display, and debugging | complete |
| 3b2-b5k | retained `+`, `-`, and `<=>` Function values, classification, binding chains, unary/product application, direct operation lowering, display, and debugging | complete |
| 3b2-b5l | specialized private scalar Function inputs for retained named/symbolic values, exact tag signatures, direct lowering, and GDB-visible parameters | complete |
| 3b2-b5m | inferred non-capturing anonymous Function values, direct and private-parameter application, call-site classifier inference, left-to-right symbolic chains, private direct lowering, display, and debugging | complete |
| 3b2-b5n | one scalar packaged operand, full positional or declaration-order labeled-prefix supply, trailing closed defaults, exact flat private signatures, and field-level debugging | complete |
| 3b2-b5o | root-scope nominal labeled Union and positional Variant construction, private aggregate function passage, exhaustive payload decisions, canonical display, and active-payload debugging | complete |
| 3b2-b5p | unpublished non-escaping nested lexical functions, represented immutable capture snapshots, exact private capture parameters, direct calls, and GDB-visible nested frames/captures | complete |
| 3b2-b5e8 | remaining function forms, Function results/aggregate boundaries, remaining symbolic and capturing/escaping anonymous Function values, anonymous product patterns, general packaged operands/default scopes, return-through-block cleanup, remaining recursion/totality evidence, escaping/recursive/overloaded nested functions and callable/Scope/evidence captures, persistent aggregate storage, recursive sums, constraints, capabilities, and decisions | planned |
| 4a | prospective UTF-8 String byte count through the native descriptor and arbitrary-precision Int runtime | complete |
| 4b1 | exact preserved-sequence String equality and derived `Optional String` equality | complete |
| 4b2 | empty String construction, adjacent literal composition, exact concatenation, emptiness, dynamic canonical display, and GDB inspection | complete |
| 4b3a | closed Character constraint validation, retained function/equality evidence, lossless String forgetting, and debugging | complete |
| 4b3b | closed Character/entry counting and exact indexing with Optional Character passage, decisions, display, and debugging | complete |
| 4b3c | closed pinned-Unicode uppercase, lowercase, full case-fold, NFC/NFD normalization, canonical equivalence, and debugging | complete |
| 4b3d-a | contextual immutable `List Effect` construction, private pointer passage, canonical display, process-lifetime allocation, and debugging | complete |
| 4b3d-b | contextual immutable `List Int` construction, exact entry/sequence/subsequence containment, private passage, display, and debugging | complete |
| 4b3d-c | immutable exact `List Int` remove-first/remove-all with source-order retention, selective sharing/rebuilding, and conditional runtime fragments | complete |
| 4b3d-d | basic immutable `List Int` empty/singleton construction, insertion/concatenation/reversal, observations, structural equality, total decomposition, Optional projections, and debugging | complete |
| 4b3d | dynamic Character and Unicode operations, remaining strings and container types, remaining List ordering/algorithms/traversal/transformation, and representation-safe reclamation | planned |
| 5 | generators, suspension, closure environments, linear close/resume behavior | planned |
| 6a | executable `root` Scope identity and direct qualified ordinary/static root-function calls | complete |
| 6b1 | source-root function namespace aliases, typed Scope aliases, alias chains, declaration snapshots, and qualified overload preservation | complete |
| 6b2a | stable source-root and alias data members, alias-chain declaration snapshots, typed Scope lookup, and published root bindings within one application | complete |
| 6b2b1 | root/root-alias Scope parameters specialized with captured overload metadata and typed private data-environment forwarding | complete |
| 6b2b2 | direct function-body root data, Scope results/escape, nested qualified Scope members, generator members, non-root aliases, multi-component/external `use`, published module/package/application interfaces, source and compiled libraries, GEIR instantiation, incremental and link-time compilation | planned |
| 6b3a | source-root `use` of the live root or a retained root alias, immutable snapshot preservation, direct qualified members, runtime erasure, and debugging | complete |
| 6c1 | direct-entry scalar defining-context capture with declaration filtering, explicit private parameters, lexical-shadow isolation, and GDB observation | complete |
| 6c2 | cross-function capture forwarding, aggregate/callable environments, anonymous and escaping closures, qualified root access, and public/library context ABI | planned |
| 7a | canonical empty first-class Effect values, classification, equality, decomposed products, scalar function passage, display, and debugging | complete |
| 7b | nonempty effects and inference, resources, layouts, locations, tasks, deterministic scheduling, transactions, time, static flow, and platform packages | planned |
| 8a | closed fundamental Type values, canonical identity equality/display, scalar function passage, and debugging | complete |
| 8a1 | root named Constraint objects over primitive bases, checked closed Boolean predicates, classified-copy identity, private display tags, and debugging | complete |
| 8a2 | closed Int-constraint proof/rejection, dynamic predicate evaluation, refined base operations, existing Result/Error integration, and debugging | complete |
| 8b | user-defined Type values, native serialization, introspection, contracts/evidence, implementation plans, information flow, and remaining `v0.2` assurance behavior | planned |
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
representation. Increment 1a retains the shared diagnostic-control validation
and erases valid controls before LLVM because they have no runtime semantics;
future compiler warnings must consult their source-scoped identities at
publication. Increment 2a removes the integer boundary with a freestanding Topal
numeric runtime and a private, dynamically sized representation. Increment 2b
adds finite exact division and Rational values. Increment 2c-a adds the fully
normative Comparison decision forms; 2c-b1 adds the initial typed arithmetic
Result ABI and failure paths, while 2c-b2 and 2c-b3 retain narrowing,
projection, observation, and Result decisions. Increment 2c-b4 exposes the same
closed nominal arithmetic-code vocabulary as direct qualified values.
Increment 2c-b5 completes the five structured Error observations by wrapping
nullable detail/cause fields and materializing present one-based source
locations through the existing private Optional ABI. Increment 2c-c retains
the infinity work that requires normative completion. Increment
2d-a adds the fully normative explicitly bounded finite range subset; 2d-b
closes unbounded and infinite endpoints after increment 2c-c.
Range-based collection selection remains grouped with containers. Increment 2e
reuses the validated Nat value as its exact Int representation for comparison,
without an unsigned conversion or a second numeric runtime. Increment 2f
retains modular nominal identity while reusing that same exact Int carrier and
reduces after each wrapping operation, so no machine-width overflow or
optimization determines semantics. Dynamic checked construction and named
range operands are completed by increment 2f1 using static evidence where
conclusive and exact generated bound comparisons otherwise. The resulting
private Result reuses the established arithmetic Error representation while a
layout-compatible specialized DWARF header keeps its modular success type
reachable to GDB. Increment 3a admits the
statically decidable scalar overload, complete-header acyclic forward calls,
and basic static-function foundation. Increment 3b1 adds the first root-scope
user-declared nominal value
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
state; 3b2-b5e5 closes overload-specific recursion identity; 3b2-b5e6 admits
complete uniform mutual `Int` cycles; 3b2-b5e7 composes that closure with
range-preserving `Nat` evidence; 3b2-b5f adds recursively composed private
Tuple results, delegates physical call lowering to LLVM, and retains aggregate
locals in target-exact DWARF through debug-only stack shadows; 3b2-b5g extends
the decomposed representation through every admitted decision family with
field-wise scalar `phi` joins; 3b2-b5h adds exact private Tuple parameters with
interpreter-compatible unary-versus-multi-parameter call normalization and
GDB-visible aggregate arguments; 3b2-b5i gives structural Records a private
canonical-field carrier plus construction-order permutation across functions,
decisions, nesting, display, and debug shadows; 3b2-b5j lets named Function
values retain compile-time overload identity and direct calls; 3b2-b5k retains
direct symbolic `+`, `-`, and `<=>` identities; 3b2-b5l specializes scalar
Function inputs while retaining exact private tags for debugging; 3b2-b5m
specializes inferred non-capturing anonymous functions at their private call
sites without choosing a closure ABI; 3b2-b5n normalizes one closed scalar
operand package to an exact flat private call boundary; 3b2-b5o adds nominal
labeled and positional sums with statically typed private aggregate payload
slots, LLVM-owned physical call lowering, exact active-alternative decisions,
and active-only debugging; 3b2-b5p closure-converts directly applied nested
lexical declarations into exact private capture parameters without materializing
a closure value; and 3b2-b5e8 continues with escaping, recursive, overloaded,
and cross-callable nested functions, remaining recursive and nested-block exit
control flow, cleanup-bearing scopes, persistent aggregate storage, and the
remaining user-defined value representations.
Increment 4a admits the encoding-observation
byte count without
attaching an encoding or importing a foreign String runtime; 4b1 adds exact
preserved-sequence equality without normalization or locale policy; and 4b2
adds empty construction, literal composition, exact dynamic concatenation,
emptiness, and canonical dynamic display. Increment 4b3a retains statically
proved Character evidence over the same descriptor; 4b3b adds closed
observations under pinned segmentation; 4b3c adds closed pinned-Unicode
transformations and canonical equivalence; 4b3d retains dynamic observations
and the remaining Unicode and container work, while 4b3d-a establishes the
private immutable `List Effect` node and function-boundary foundation without
claiming a generic, persistent, or public representation. Increment 4b3d-b
adds the exact Int payload specialization and allocation-free containment laws
without changing that boundary. Increment 4b3d-c adds exact value removal with
selective immutable sharing and Topal-owned reconstruction, while independently
conditional runtime fragments avoid adding work to unrelated modules.
Increment 4b3d-d adds the remaining basic `List Int` constructors,
observations, structural equality, total decomposition, and immutable
composition/reversal while preserving present empty tails through the tagged
Optional representation.
Increment 6a resolves the executable root
Scope identity and direct qualified root functions entirely in
the frontend; 6b1 retains source-root function snapshots, typed aliases, alias
chains, and overload order without a namespace runtime; 6b2a adds stable
entry-frame data identities and exact alias snapshots without initializer
re-execution; 6b2b1 specializes a root/root-alias Scope argument into retained
overload facts plus a typed private data environment that can be forwarded
without caller-frame lookup; 6b3a makes the live root or a retained root alias
available without flattening by returning the same static snapshot and erasing
`use` before LLVM; 6c1 closure-converts direct-entry scalar `@ member`
selections to explicit private capture parameters; and 6b2b2/6c2 retain direct
cross-function root storage, Scope escape, nested Scope and generator members,
external path resolution, package construction, published interfaces, and
source/compiled-library work.
Increment 7a admits the inert
canonical empty `Effect` as a distinct zero-data value; 7b retains effect
execution/inference and the remaining platform-semantic work. Increment 8a
admits the closed fundamental `Type` identities without runtime reflection;
8a1 adds closed named Constraint-object metadata and private observation tags;
8a2 applies closed Int constraints, retains static evidence over unchanged base
storage, and reuses the existing Result/Error path for dynamic validation. 8b
retains open-world type metadata, general constraint evidence/application, and
the remaining assurance work.
