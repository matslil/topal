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
| 4b3d-e | contextual `List Int` map/select/Int-state fold specialized as finite LLVM loops with immutable captures, exact ordering, and debugging | complete |
| 4b3d-f | bound anonymous Function values consumed by `List Int` map/select/fold through snapshot-preserving direct specialization, with retained Function debugging and no dispatch ABI | complete |
| 4b3d-g | exact `Range Int` value/index selection over `List Int` plus closed user-perceived-Character index selection over String, without observable slice storage | complete |
| 4b3d-h | `Continue Int`/`Finish Int` construction and short-circuiting `List Int` fold control through direct LLVM branches, private storage, and semantic debugging | complete |
| 4b3d-i | contextual `List (Int, Int)` construction and anonymous product-pattern map to `List Int` with inline pair nodes and direct field binding | complete |
| 4b3d-j | exact `List (Int, String)` and `List List (Int, String)` construction, private outer passage, first/count/equality, nested display, and debugging | complete |
| 4b3d-k | closed direct String Character foreach through pinned segmentation and ordered inline Unit actions, with Character debugging and no Generator runtime | complete |
| 4b3d-l | exact named `Generator Character Unit Unit` construction, explicit classification, compiler-held sequence provenance, one local foreach transfer, consumed-use rejection, and Generator debugging | complete |
| 4b3d | dynamic Character and Unicode operations, remaining strings and container types, remaining List ordering/sequence/index/traversal algorithms and callable forms, traversal-control generalization, and representation-safe reclamation | planned |
| 5a | qualified `lang generator generator-closed` as a closed nominal value with equality, display, and debugging but no continuation behavior | complete |
| 5b | lazy `Generator Int Unit Unit` construction from `iterate` and direct `take-while`, with checked dormant bodies, one-consumption local linearity, canonical observation, and debugging but no traversal | complete |
| 5c | direct bounded Int iterate collection through an ordered generated SSA/List loop, with exact stopping behavior and result debugging but no Generator object or generic runtime | complete |
| 5d | lazy `Generator Int Unit Unit` construction from a `List Int` seed and an exact `unfold` step, preserving distinct seed/yield types, one-consumption local linearity, canonical observation, and debugging but no traversal | complete |
| 5e | finite collection of a locally retained `List Int`/`uncons` unfold through an ordered seed/List SSA loop, with no Optional or Generator object/runtime | complete |
| 5f | root bounded Int `iterate` foreach with exact predicate/body/next order, Unit result, iteration debugging, and no allocation or Generator object/runtime | complete |
| 5g | root single-yield custom `Generator Character Unit Unit` declaration, application, suspension, local foreach consumption, and debugging through ordered inline lowering | complete |
| 5h | consecutive custom Character yields with exact action/resume ordering, finite inline expansion, and no continuation object or runtime | complete |
| 5i | one exact generator-local Character alias retained across admitted yields, caller non-escape, and lexical DWARF inspection without semantic state storage | complete |
| 5j | exact custom Character generator completion before its first yield, zero action invocations, and direct final Unit without continuation state | complete |
| 5k | one yielded Character followed by a distinct exact final Character, ordered result materialization, direct root observation, and full-direction debugging | complete |
| 5l | exact generator-local Character activation after Unit resumption, followed by a second suspension, with stage-ordered DWARF and no continuation state | complete |
| 5m | exact successful Unit resume-result binding after one custom yield/action, used as final Unit with source-lifetime debugging and no continuation state | complete |
| 5n | exact function-local custom Generator abandonment with checked close ordering, erased handler-free completion, and function-scope debugging | complete |
| 5o | exact function-local custom Generator close handling with nominal close Result/Error materialization, statically selected Error action, and generator-code debugging | complete |
| 5p | exact qualified `generator-closed` handler rule with nominal static selection, inactive fallback binding, and source-level debugging | complete |
| 5ak | exact successful `Result Rational` input/yield, reflexive action, resumption, and structured division-error final with pointer debugging | complete |
| 5al | exact nominal `Comparison` input/yield, equality action, resumption, and distinct Greater final with ordered scalar debugging | complete |
| 5am | exact recursive `Optional (Int, String)` input/yield, structural equality action, resumption, and distinct Some final with boxed-payload debugging | complete |
| 5an | exact successful recursive `Result ((Int, String), ArithmeticErrorCode)` input/yield, structural equality action, resumption, and distinct successful final with boxed-payload debugging | complete |
| 5ao | exact absent recursive `Optional (Int, String)` input/yield, tag equality action, resumption, and absent final with full-classifier debugging | complete |
| 5ap | exact Boolean input/yield/action and post-resume Boolean decision selecting a distinct final String with branch and value debugging | complete |
| 5aq | exact recursive nominal `(Optional Choice, Result (Choice, ArithmeticErrorCode))` input/yield, guarded structural action, resumption, and distinct final alternatives with complete recursive debugging | complete |
| 5ay | complete closed ordered `List Int` sequence vocabulary, contextual `List String`, checked positions, explicit zip policies, ordered traversal/removal/collection, private immutable lowering, and semantic debugging | complete |
| 5ax | exact `Generator List Int Unit List Int` construction, function-result and function-parameter ownership transfer, count-before-append ordering, private target-derived calls, and complete recursive List debugging | complete |
| 5aw | exact `Generator Optional (Int, String) Unit Result ((Int, String), ArithmeticErrorCode)` construction, function-result and function-parameter ownership transfer, tag-gated field action, private target-derived calls, and complete recursive-value debugging | complete |
| 5av | exact `Generator (Int, String) Unit (Int, String)` construction, function-result and function-parameter ownership transfer, field-ordered action and result, private target-derived aggregate calls, and complete debugging | complete |
| 5au | exact `Generator Int Unit String` construction, function-result and function-parameter ownership transfer, typed traversal result, private target-derived calls, and complete debugging | complete |
| 5at | source-ordered unary and positional-product custom Generator overload selection, ordered multi-input capture, distinct yield directions, and typed foreach final-result bindings with complete debugging | complete |
| 5aj | exact `(Int, String)` input, yield, equality action, resumption, and distinct final product with ordered aggregate debugging | complete |
| 5ai | exact nominal `Choice` input, yield, equality action, resumption, and distinct final alternative with ordered private-value debugging | complete |
| 5ah | exact `Nat` input, yield, increment action, resumption, and incremented final with ordered private-value debugging | complete |
| 5ag | exact `Range Int` input, yield, membership action, resumption, and narrowed final intersection with ordered private-value debugging | complete |
| 5af | nominal `Optional Int` input, yield, equality action, resumption, and final directions with ordered private-value debugging | complete |
| 5ae | payload-free Unit input, yield, identity action, resumption, and final directions with ordered debug-only lifetime anchors | complete |
| 5ad | exact canonical Rational input, yield, action, resumption, and final directions with ordered allocation-aware construction/addition and debugging | complete |
| 5ac | exact arbitrary-precision Int input, yield, action, resumption, and final directions with ordered allocation-aware additions and debugging | complete |
| 5ab | independent Boolean input, yield, and final directions with ordered action/resumption, private `i1`, and source-level debugging | complete |
| 5aa | exact explicit final String after one String yield/action and Unit resumption, with separate return provenance and debugging | complete |
| 5z | exact explicit final String return before any suspension, with zero action invocations and return-site debugging | complete |
| 5y | one typed discarded String computation after Unit resumption and before the next suspension | complete |
| 5x | exact distinct final String after String yield/action and Unit resumption | complete |
| 5w | independent String yield direction with ordered initial/literal suspensions and String action debugging | complete |
| 5v | independent String initial input with an ordered emptiness prefix before one custom Character suspension | complete |
| 5u | exact custom Generator function result with distinct final Character preserved through caller traversal | complete |
| 5t | exact custom Generator parameter traversal with distinct final Character preserved as the callee result | complete |
| 5s | exact unconsumed custom Generator parameter close with retained provenance and O0-erased handler-free delivery | complete |
| 5r | exact custom single-yield Generator function parameter traversal with private ownership/provenance transfer and parameter debugging | complete |
| 5q | exact custom single-yield Generator function result with private ownership/provenance transfer and return-value debugging | complete |
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
| 7a1 | explicit empty ordinary-function effect bounds with checked containment, retained static Function-view metadata, and pre-LLVM erasure | complete |
| 7b1 | closed external-layout policy values as seven nominal families, canonical display, same-family equality, and debugging without layout construction or authority | complete |
| 7b | nonempty effects and inference, resources, layout construction and encoding, locations, tasks, deterministic scheduling, transactions, time, static flow, and platform packages | planned |
| 8a | closed fundamental Type values, canonical identity equality/display, scalar function passage, and debugging | complete |
| 8a1 | root named Constraint objects over primitive bases, checked closed Boolean predicates, classified-copy identity, private display tags, and debugging | complete |
| 8a2 | closed Int-constraint proof/rejection, dynamic predicate evaluation, refined base operations, existing Result/Error integration, and debugging | complete |
| 8a3 | closed fundamental-Type identity/view/relations, exact initial language context, numeric Version value, static erasure, and debugging | complete |
| 8a4 | closed atomic Capability values, canonical conjunction/alternatives, root static binding chains, literal final observation, and complete runtime/debug erasure | complete |
| 8a5 | source-root v0.1 function-interface shapes, exact intentional implementation evidence, direct ordinary calls, complete runtime/debug erasure, and truthful implementation debugging | complete |
| 8b | user-defined Type values, native serialization, general introspection and context changes, contracts/evidence, implementation plans, information flow, and remaining `v0.2` assurance behavior | planned |
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
Increment 4b3d-e specializes contextual anonymous map, select, and Int-state
fold bodies into finite source-ordered loops without introducing Function
objects, callback ABIs, indirect calls, or a collection dispatcher.
Increment 4b3d-f resolves an earlier bound anonymous Function's retained body
and definition-time capture snapshot into those same loops while keeping its
private tag solely for source identity and debugging.
Increment 4b3d-g reuses exact Range membership in a conditional immutable List
selection fragment and evaluates closed String index slices with the shared
pinned user-perceived-Character segmentation. Dynamic String slicing remains
deferred until it can use a freestanding Topal Unicode runtime rather than a
foreign library.
Increment 4b3d-h gives `Continue Int` and `Finish Int` a private immutable
two-word representation and branches on it directly inside the specialized
`List Int` fold. A Finish payload reaches the fold result without evaluating
later actions; the representation remains outside ordinary function and
library ABI boundaries.
Increment 4b3d-i admits the first product-element List only where an anonymous
two-field pattern maps `List (Int, Int)` to `List Int`. Inline three-word nodes
and direct field loads preserve exact values without a Tuple allocation,
generic List runtime, callback, or pair-List function ABI.
Increment 4b3d-j composes exact private two-word outer nodes with three-word
`(Int, String)` inner nodes. It admits direct outer function passage, first,
count, equality, canonical nested output, and semantic debugging through
shape-specific nonrecursive loops, while leaving inner boundaries and a
generic or compiled-library List ABI closed.
Increment 4b3d-k evaluates a closed direct `characters` source with the pinned
segmentation already used by Character observations and expands its
capture-free Character-to-Unit action in preserved order. Existing String
descriptors and a debug-only pointer shadow preserve Character inspection
without a Generator object, continuation token, callback, indirect call, or
foreign Unicode/runtime dependency. Dynamic Strings, function-transferred
generators, and captured actions remain in the planned Unicode and generator
closures.
Increment 4b3d-l retains the same closed Character sequence beside an exact
classified root Generator binding, evaluates its String once at construction,
and transfers the provenance into one foreach before rejecting later source
use. A compiler-private semantic token supports observation and Generator
DWARF without becoming cursor state or a copyable continuation. Root
abandonment, general/dynamic function results, nonspecialized parameter
traversal, general close behavior, and library representation remain in the
planned generator closure.
Increment 4b3d-m consumes a direct closed `characters` traversal into String by
retaining its pinned cluster sequence as checking evidence and forwarding the
once-evaluated immutable source descriptor. This required O0 identity lowering
constructs no intermediate List or Generator and introduces no concatenation,
Unicode, or foreign runtime; dynamic, transformed, stored, and library
traversals remain in the planned closure.
Increment 4b3d-n transfers one closed Character generator into a private
single-parameter Unit function and records implicit owned close at its
statement-free Unit exit. LLVM `fastcc` selects the physical AMD64 argument
placement; generated close needs no runtime action because the specialization
owns no continuation state. A debug-only token shadow preserves GDB inspection,
while traversal outside the next specialization, return, general close, and
public/library ABI remain planned.
Increment 4b3d-o transfers retained Character provenance from a top-level call
into a distinct private instance of a one-parameter Unit function whose sole
executable body is capture-free foreach. The caller evaluates its String once
and LLVM `fastcc` transfers the private ownership token; the callee expands the
exact ordered Characters, exhausts the continuation, and returns Unit.
Compile-session provenance is restored after checking, nested calls remain
closed, and no Generator runtime or public ABI is introduced.
Increment 4b3d-p retains the pinned Character sequence from a statement-free
`characters` result beside each distinct private function symbol. A top-level
exact String call passes the existing descriptor with LLVM `fastcc`; the callee
returns the private `i32` token, and the caller binds and consumes it once. GDB
observes the String parameter and semantic Generator return, while the
compile-session side table creates no runtime state, serialized metadata, or
public ABI.
Increment 5a admits only the qualified `generator-closed` vocabulary value as a
private nominal enum. It deliberately creates no continuation, generator state,
close result, Error domain, or provenance; all generator execution remains in
increment 5. Increment 5b admits the first exact Generator value through lazy
`iterate` construction and directly chained `take-while`. It checks and retains
the anonymous bodies without invoking them, evaluates the initial value once,
and enforces a deliberately narrow one-consumption local boundary around a
private debug token; traversal, close delivery, executable continuation state,
and a compiled-library Generator ABI remain in increment 5. Increment 5c
specializes direct bounded Int iterate collection as explicit LLVM control flow
and immutable List construction. It tests before publishing, advances only an
accepted candidate, and keeps closure captures, indirect Generator operands,
foreach, unfold traversal/collection, custom generators, resumable state, and
library representation in increment 5. Increment 5d admits exact lazy `unfold`
construction whose `List Int` seed and Int yield types are deliberately
distinct. It checks and retains the unary step without invoking it, evaluates
the seed once, reuses the construction-only debug token and local linearity
boundary, and leaves traversal plus executable state to increment 5. Increment
5e specializes consumption of that value, including linear local moves, when
the seed is an already evaluated immutable `List Int` binding and the step is
exactly parameter `uncons`. A direct LLVM seed/List loop copies yielded heads
in order and terminates at `Empty` without Optional materialization, callback,
Generator object, or generic runtime; arbitrary unfold steps and general
resumable state remain in increment 5. Increment 5f specializes root
`foreach` over a locally retained bounded Int iterate whose literal initial
value, next, predicate, and Unit action need no captures. The explicit LLVM loop
tests before visiting, advances only after the Unit action, returns Unit at the
first rejection, and allocates no collection or Generator object. Dynamic
initials, captures, other classifiers, and general traversal remain in
increment 5. Increment 5g admits the first custom suspension boundary: an exact
root `Generator Character Unit Unit` declaration that yields its sole Character
input once and then returns Unit. Checking retains declaration and yielded-value
provenance, while Linux x86-64 lowering evaluates the input once and expands the
proven suspension, Unit action, resumption, and final Unit in source order. The
private observation token and DWARF identities require no continuation object,
state allocation, callback, dispatcher, foreign runtime, or public Generator
ABI. At 5g, multiple yields, dynamic provenance, captures, close handling, and
general state machines remained in increment 5. Increment 5h generalizes this
exact proof to one or more consecutive yields of the same initial Character.
The checked model retains the ordered finite suspension sequence, while the
backend emits one direct action block and erased Unit resumption per yield
before final Unit.
This ordering is mandatory at O0 and does not depend on LLVM unrolling. Ordinary
inter-yield statements and distinct or dynamic yielded values remained in
increment 5. Increment 5i admits one explicitly classified Character alias of
the initial parameter before those yields. Its checked identity never enters
the caller environment, and a generator lexical DWARF shadow makes the exact
preserved value inspectable during traversal without becoming semantic
continuation storage. Other local computation, captures, close handling, and
general continuation state remained in increment 5. Increment 5j admits the
empty suspension sequence whose exact body returns Unit immediately. The
checked action is invoked zero times, and O0 lowering produces final Unit
directly without action code or debug storage. Non-Unit final results, explicit
early returns, captures, close handling, and general continuation state remained
in increment 5. Increment 5k admits one distinct closed Character final after
one yielded Character. The frontend retains separate yield and result
provenance and emits the final descriptor only after the direct action and Unit
resumption. Dynamic finals, result binding, other directions, captures, close
handling, and general continuation state remain in increment 5.
Increment 5l admits one explicitly classified immutable Character alias after
one or more yielded initial Characters and before one or more yields of the
alias. The checked model records the successful-resumption count at which the
local activates; O0 lowering therefore completes the prefix actions and Unit
resumptions before introducing the local DWARF shadow and continuing to the
next suspension. This stage evidence remains compiler-private and creates no
runtime program counter, continuation layout, foreign dependency, or native
ABI change. General generator body computation, mutable or multiple locals,
resume bindings, close paths, captures, and external boundaries remain in
increment 5.
Increment 5m admits one named successful Unit result from a custom Character
yield and uses it as the generator's final Unit. The checked activation stage
occurs only after the direct action and Unit resumption; O0 lowering represents
the binding with a debug-only `i8` shadow and keeps it distinct from the
unimplemented `generator-closed` edge. No semantic continuation state, foreign
dependency, public Generator layout, or native ABI revision is introduced.
General resume classifiers, additional body execution, close handling,
captures, and external boundaries remain in increment 5.
Increment 5n admits one call-specialized function-local instance of the exact
single-yield custom Character generator and closes it at function scope exit.
The checked block records close before final Unit; because the generator has no
handler, cleanup, effect, or post-yield work, native O0 lowering erases the
expected close value while preserving ownership and Generator DWARF identity.
Handled close paths, multiple owners, Generator transfer, dynamic provenance,
and library boundaries remain in increment 5.
Increment 5o admits the adjacent bound-yield close handler with complete Error
and Ok Unit branches. The checked model retains both branches and their binding
activation, while known abandonment materializes the nominal
`generator-closed` failure and executes only its Error action. Generator-specific
Result/Error DWARF preserves the code-set identity in GDB. Qualified code
matching, non-Unit cleanup/effects, successful traversal through this handler,
multiple owners or yields, transfer, and library boundaries remain in increment
5.
Increment 5p admits one qualified `lang generator generator-closed` rule before
the generic Error fallback. The checked model retains the nominal matcher and
all branch metadata, while the statically known close edge executes only the
qualified Unit action without a runtime switch or fallback-binding activation.
Other code sets and handler forms, dynamic close results, multiple owners or
yields, transfer, and library boundaries remain in increment 5.
Increment 5q admits one statement-free ordinary Character factory returning a
fresh exact single-yield custom Generator. Call-specialized checked provenance
and LLVM `fastcc` transfer ownership into one caller traversal without close or
public continuation state; Character-parameter and Generator-return DWARF remain
inspectable through the boundary. Parameter transfer, other factory shapes,
dynamic/general continuation results, and libraries remain in increment 5.
Increment 5r admits one root-owned exact single-yield custom Generator argument
to a call-specialized ordinary consumer whose only executable body is traversal
of that parameter. The caller binding is consumed, checked provenance is mapped
to the sole callee owner, and LLVM `fastcc` carries only a private `i32` token;
the callee expands the retained Character action and Unit completion directly.
Unconsumed-parameter close, other states or directions, nested/general transfer,
and libraries remain in increment 5.
Increment 5s admits the same transferred parameter when its Unit body leaves the
continuation unconsumed. The checked model records an explicit custom close with
the call-specialized construction provenance and lexical root domain. Since the
exact suspension has no handler, state, cleanup, effects, or following work, O0
lowering erases delivery and final Unit while preserving ownership and Generator
DWARF. Handled close, richer state, nested/general transfer, and libraries remain
in increment 5.
Increment 5t extends the transferred custom parameter to the complete
`Generator Character Unit Character` classifier. The callee's result-valued
foreach expands the retained yielded Character action and Unit resumption before
returning the separately retained final Character through the ordinary private
function result. LLVM `fastcc` selects both token-parameter and descriptor-return
placement; Generator, yielded Character, final Character, and both frames remain
debuggable without a continuation object, state machine, foreign runtime, or
public Generator ABI. Other classifiers and richer state, bodies, transfer, and
library boundaries remain in increment 5.
Increment 5u admits the matching complete classifier as a fresh ordinary
function result. A private-symbol provenance entry transfers the exact
declaration, yielded and final Characters, suspension graph, and ownership to
the caller while the factory returns only an LLVM `fastcc` `i32` token. Caller
traversal expands the action and Unit resume before its distinct final Character;
factory and caller values remain debuggable without public continuation state,
a foreign runtime, or a Generator ABI. Richer factories, composed transfers,
other classifiers, general state, and libraries remain in increment 5.
Increment 5v separates the initial direction by admitting one root custom
`Generator Character Unit Unit` with a String input and a checked
`empty? initial` Boolean binding before suspension. Application evaluates the
existing String descriptor once and executes that Topal-owned operation before
exposing the private token; traversal then performs the exact Character action
and Unit resume/final path. Initial, prefix-binding, Generator, yielded-Character,
and entry-frame debug evidence remains available without a continuation object,
foreign runtime, other-language standard library, public ABI, or native-layout
revision. Other shapes, directions, transfers, state, and libraries remain in
increment 5.
Increment 5w separates the yield direction by admitting one root custom
`Generator String Unit Unit`. Application evaluates its String input once;
traversal reuses that descriptor for an initial-value suspension, materializes
the following exact String literal at its own suspension, and performs each
String action and Unit resume before final Unit. The private `i32` token remains
debug/ownership evidence rather than continuation state. Generator and both
yielded Strings remain inspectable without a foreign runtime, other-language
standard library, public ABI, or native-layout revision. Computed yields, body
state, other directions, transfer, close, and library boundaries remain in
increment 5.
Increment 5x preserves a distinct exact final String through
`Generator String Unit String`. The checked graph separates that final
expression from the once-evaluated initial String, yield provenance, and Unit
resume edge. O0 traversal performs the String action and resumption before
materializing and returning the final descriptor, which becomes the root output.
The full classifier, Generator value, yielded String, final source position, and
entry frame remain debuggable without a continuation object, foreign runtime,
other-language standard library, public ABI, or native-layout revision.
Nonliteral finals, other directions, body state, transfer, close, and libraries
remain in increment 5.
Increment 5y retains one exact ordinary discarded computation between String
suspensions. The checked graph records its typed continuation block and
successful-resumption ordinal separately from the initial value and ordered
yields. O0 traversal invokes the first action, resumes with Unit, evaluates
`empty? initial`, and only then reaches the next suspension and action. The
Generator, yielded Strings, captured initial String, continuation source site,
and entry frame remain debuggable without a continuation object, foreign
runtime, other-language standard library, public ABI, or native-layout
revision. Bindings, additional or differently placed computations, other
directions, non-Unit finals, transfer, close, and libraries remain in increment
5.
Increment 5ak separates Result input, yield, and final directions through one
exact custom Generator of `Result Rational` values. The checked graph retains
the nominal Result/success/error identities, exact Rational 1 success evidence,
initial-parameter yield, proved self-equality action, Unit resumption, final
source-located division failure, declaration provenance, and ownership edge.
O0 traversal reuses the once-promoted success for the action, resumes, and only
then constructs the structured division-by-zero Error. Two debug-only aligned
pointer shadows and four source-anchor stores keep yield, action, captured
initial, and final lines inspectable. Existing Topal-owned Result/Rational/Error
storage, display, and Linux syscall writer are reused without generic inlined
Result propagation, a semantic Generator runtime, foreign dependency, other-
language standard library, public ABI, or native-layout revision. Other Result
identities, values, shapes, directions, state, transfer, close, and libraries
remain in increment 5.

Increment 5al separates the language-defined nominal `Comparison` identity in
all three directions of one exact `Generator Comparison Unit Comparison`.
Application evaluates `1 <=> 2` once; traversal reuses that Less value for the
yield, independently evaluates the action comparison and nominal equality,
resumes with Unit, and only then evaluates `3 <=> 2` to return Greater. Two
debug-only aligned `i32` shadows and four source-anchor stores keep yield,
action, captured initial, and final lines inspectable. Existing Topal-owned
arbitrary-precision Int comparison, Comparison equality/display, and Linux
syscall writer are reused without a semantic Generator runtime, foreign
dependency, other-language standard library, public ABI, or native-layout
revision. Other Comparison expressions, values, directions, state, transfer,
close, and libraries remain in increment 5.

Increment 5am recursively composes the existing Optional and positional-product
representations through one exact
`Generator Optional (Int, String) Unit Optional (Int, String)`. Application
constructs `Some (7, "item")` once; traversal reuses it for the yield, constructs
the exact action operand, compares tags before fields, resumes with Unit, and
only then constructs `Some (8, "done")`. Each Some payload is a Topal-owned
aligned 16-byte pair of the existing Int/String pointers. Existing Optional
headers, field equality/display, two pointer debug shadows, DWARF recursive type
names, the Topal GDB renderer, allocator, and Linux syscall writer are reused
without a semantic Generator runtime, foreign dependency, other-language
standard library, public ABI, or native-layout revision. Other recursive
Optional values, directions, state, transfer, close, and libraries remain in
increment 5.

Increment 5an recursively composes the existing Result and positional-product
representations through one exact successful
`Generator Result ((Int, String), lang arithmetic ArithmeticErrorCode) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)`.
Application constructs the implicit success `(7, "item")` once; traversal
reuses it for the yield, constructs the exact action success operand, checks
success tags before fields, resumes with Unit, and only then constructs the
implicit success `(8, "done")`. Each success payload is a Topal-owned aligned
16-byte pair of the existing Int/String pointers. Existing Result headers,
field equality/display, two pointer debug shadows, DWARF recursive type names,
the Topal GDB renderer, allocator, and Linux syscall writer are reused without
a semantic Generator runtime, foreign dependency, other-language standard
library, public ABI, or native-layout revision. Errors and other recursive
Result values, directions, state, transfer, close, and libraries remain in
increment 5.

Increment 5ao preserves the recursive `Optional (Int, String)` classifier
through one exact all-None Generator graph. Application constructs one absent
Optional; traversal reuses it for the yield, constructs the exact absent action
operand, compares tags without observing payload storage, resumes with Unit,
and only then constructs the absent final. Existing Optional headers, tag
equality/display, two pointer debug shadows, DWARF recursive type names, the
Topal GDB renderer, allocator, and Linux syscall writer are reused without a
product-payload allocation, semantic Generator runtime, foreign dependency,
other-language standard library, public ABI, or native-layout revision. Mixed
Some/None graphs and other recursive Optional values, directions, state,
transfer, close, and libraries remain in increment 5.

Increment 5ap preserves an exact `Generator Boolean Unit String` across one
initial-Boolean suspension and a post-resume complete Boolean decision. The
once-evaluated true input reaches the action, Unit resumption precedes the
decision, and the selected `"accepted"` branch becomes the final root String.
LLVM O0 retains the direct conditional branch, two String-producing blocks,
and pointer join. Existing Boolean/String lowering, an aligned debug-only `i1`
shadow, Generator and captured-value DWARF, the Topal GDB renderer, allocator,
and Linux syscall writer are reused without a semantic Generator runtime,
foreign dependency, other-language standard library, public ABI, or native-
layout revision. Other decision subjects, matchers, actions, results, generator
shapes, state, transfer, close, and libraries remain in increment 5.

Increment 5aq recursively preserves one declared `Choice is Enum (First,
Second)` through the Optional and arithmetic Result fields of an exact
`Generator (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)) Unit (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))`.
Application constructs `(Some First, First)` once; traversal reuses it for the
yield, constructs the exact action operand, compares Optional and Result tags
before loading boxed Choice tags, resumes with Unit, and only then constructs
`(Some Second, Second)` as the final/root value. Six Topal-owned four-byte enum
boxes compose with existing Optional/Result headers and the private aggregate.
Two aggregate debug shadows, specialized recursive DWARF headers, the Topal GDB
renderer, allocator, and Linux syscall writer expose the complete classifier
and values without a semantic Generator runtime, foreign dependency,
other-language standard library, public ABI, or native-layout revision. Other
recursive nominal graphs, values, directions, state, transfer, close, and
libraries remain in increment 5.

Increment 5aj separates positional-product input, yield, and final directions
through one exact custom `Generator (Int, String) Unit (Int, String)`. The
checked graph retains product arity and source order, both field classifiers,
the initial-parameter yield, field-wise equality action, Unit resumption, final
`(8, "done")`, declaration provenance, and ownership edge independently. O0
traversal reuses the once-evaluated `(7, "item")` fields for the action, resumes,
and only then materializes the final product. Two debug-only aligned aggregate
shadows and four source-anchor stores keep yield, action, captured initial, and
final lines inspectable. Existing Topal-owned Int/String storage, structural
equality/display, and Linux syscall writer are reused without a semantic
Generator runtime, foreign dependency, other-language standard library, public
ABI, or native-layout revision. Other product arities, fields, orders, shapes,
directions, state, transfer, close, and libraries remain in increment 5.

Increment 5ai separates nominal Choice input, yield, and final directions
through one exact custom `Generator Choice Unit Choice`. The checked graph
retains the Choice identity, ordered First/Second alternatives and private tags,
initial-parameter yield, exact equality action, Unit resumption, final Second,
declaration provenance, and ownership edge independently. O0 traversal reuses
the once-evaluated First tag for the action, resumes, and only then selects the
final Second tag. Two debug-only aligned `i32` shadows and four source-anchor
stores keep yield, action, captured initial, and final lines inspectable even
when LLVM folds the constant comparison and selection. The existing Topal-owned
Enum comparison/display and Linux syscall writer are reused without a semantic
Generator runtime, foreign dependency, other-language standard library, public
ABI, or native-layout revision. Other Enum declarations, alternatives, shapes,
directions, state, transfer, close, and libraries remain in increment 5.

Increment 5ah separates nominal Nat input, yield, and final directions through
one exact custom `Generator Nat Unit Nat`. The checked graph retains the Nat
refinement and underlying exact Int representation, statically proven input,
initial-parameter yield, exact `value + 1` action, Unit resumption, final
`initial + 1`, and ownership edge independently. O0 traversal reuses the
once-evaluated Nat pointer for the action, resumes, and only then performs the
final addition, with the closed `Nat 7` validation erased by proof. Two debug-
only aligned pointer shadows keep the yielded Nat and captured initial
inspectable at their source lifetimes. The existing Topal-owned arbitrary-
precision integer representation, allocator, addition, display, Linux syscall
writer, Nat DWARF identity, and GDB printer are reused without a semantic
Generator runtime, foreign dependency, other-language standard library,
unsigned machine ABI, public ABI, or native-layout revision. Other Nat
generator expressions, shapes, directions, state, transfer, close, general Nat
arithmetic, and libraries remain in increment 5.

Increment 5ag separates exact `Range Int` input, yield, and final directions
through one custom `Generator Range Int Unit Range Int`. The checked graph
retains the Range and Int endpoint classifiers, initial-parameter yield, exact
`5 in interval` action, Unit resumption, final intersection with `5 ..= 15`,
and ownership edge independently. O0 traversal constructs the input once,
preserves the yielded pointer, evaluates membership, resumes, and only then
constructs the retained bound and intersects it with the initial range. Two
debug-only aligned pointer shadows keep the yielded interval and captured
initial inspectable at their source lifetimes. The existing Topal-owned
Range/Int representation, allocator, membership, intersection, display, Linux
syscall writer, DWARF types, and GDB printers are reused without a semantic
Generator runtime, foreign dependency, other-language standard library,
public ABI, or native-layout revision. Other Range endpoints, body/action
expressions, shapes, directions, state, transfer, close, and libraries remain
in increment 5.

Increment 5af separates nominal `Optional Int` input, yield, and final
directions through one exact custom `Generator Optional Int Unit Optional Int`.
The checked graph retains the Optional classifier and Int payload classifier,
initial-parameter yield, exact `Some 7` equality action, Unit resumption, final
`None Int`, and ownership edge independently. O0 traversal constructs the
input once, preserves the yielded pointer, evaluates the action comparison,
resumes, and only then constructs and displays the final alternative. Two
debug-only aligned pointer shadows keep the yielded action value and captured
initial inspectable at their source lifetimes. The existing Topal-owned
Optional/Int representation, allocator, equality, display, Linux syscall
writer, DWARF types, and GDB printers are reused without a semantic Generator
runtime, foreign dependency, other-language standard library, public ABI, or
native-layout revision. Other Optional payloads, body/action expressions,
shapes, directions, state, transfer, close, and libraries remain in increment
5.

Increment 5ae separates payload-free Unit input, yield, and final directions
through one exact custom `Generator Unit Unit Unit`. The checked graph retains
the Unit initial, initial-parameter yield, named identity action, Unit
resumption, final Unit, and ownership edge separately. O0 traversal preserves
their order while erasing semantic payload operations because Unit has one
value; two debug-only `i8` lifetime slots keep the yield, action, captured
initial, and final sites stoppable and inspectable. LLVM owns target placement
and may transform those debug artifacts under the selected policy. Generator
and Unit values and the entry frame remain debuggable without allocation,
semantic Generator state, foreign runtime, other-language standard library,
public ABI, or native-layout revision. Other shapes, directions, state,
transfer, close, and libraries remain in increment 5.

Increment 5ad separates exact Rational input, yield, and final directions
through one exact custom `Generator Rational Unit Rational`. The checked graph
retains the initial Rational, initial-parameter yield, Unit resumption, action
and final one-third constructors/additions, and ownership edge separately. O0
traversal performs the action construction/addition before resumption and only
then performs the final construction/addition. All direct calls retain the
canonical immutable Rational/Int representations and allocation-failure path;
LLVM owns pointer placement and call lowering. Generator and exact Rational
values and the entry frame remain debuggable through Topal's GDB printer without
semantic Generator state, foreign runtime, other-language standard library,
public ABI, or native-layout revision. Other shapes, directions, state,
transfer, close, and libraries remain in increment 5.

Increment 5ac separates arbitrary-precision Int input, yield, and final
directions through one exact custom `Generator Int Unit Int`. The checked graph
retains the initial Int, initial-parameter yield, Unit resumption, action and
final exact-one additions, and ownership edge separately. O0 traversal performs
the action addition before resumption and only then performs the final addition.
Both direct calls retain the canonical immutable Int representation and
allocation-failure path; LLVM owns pointer placement and call lowering. Generator
and exact Int values and the entry frame remain debuggable through Topal's GDB
printer without semantic Generator state, foreign runtime, other-language
standard library, public ABI, or native-layout revision. Other shapes,
directions, state, transfer, close, and libraries remain in increment 5.

Increment 5ab separates Boolean input, yield, and final directions through one
exact custom `Generator Boolean Unit Boolean`. The checked graph retains the
initial Boolean, initial-parameter yield, Unit resumption, final negation,
action, and ownership edge separately. O0 traversal performs the Boolean action
before resumption and only then evaluates the final Boolean. LLVM `i1` remains
private while LLVM selects physical placement. Generator and Boolean values and
the entry frame remain debuggable without a Boolean helper, continuation
object, foreign runtime, other-language standard library, public ABI, or
native-layout revision. Other shapes, directions, state, transfer, close, and
libraries remain in increment 5.
Increment 5aa admits an exact explicit String return after one suspension. The
checked graph retains the initial-parameter yield, Unit resumption, explicit
return marker, and final String separately. O0 traversal invokes the action
once with the initial String, resumes, and only then materializes the exact
return literal. The full Generator, yielded and initial Strings, return site,
and entry frame remain debuggable without a continuation object, foreign
runtime, other-language standard library, public ABI, or native-layout
revision. Other yields, returns, directions, intervening state, transfer,
close, and libraries remain in increment 5.
Increment 5z distinguishes an exact explicit String return before the first
suspension. The checked graph retains the return marker and result separately
from its empty yield/continuation lists. Application still evaluates the String
input once; O0 traversal invokes no action and directly materializes the final
String. The full Generator, captured initial String, return site, and entry frame
remain debuggable through a nonsemantic stack shadow without a continuation
object, foreign runtime, other-language standard library, public ABI, or
native-layout revision. Implicit zero-yield finals, nonliteral or after-yield
returns, other directions, transfer, close, and libraries remain in increment
5.
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
canonical empty `Effect` as a distinct zero-data value; 7a1 admits the same
empty row as an explicit ordinary-function upper bound and retains its narrow
Function view only in compiler memory before erasing it ahead of LLVM. 7b1
admits the seven closed external-layout policy families as ordinary nominal
values without constructing layouts, touching external memory, or granting
authority. 7b retains nonempty effect execution/inference, layout construction
and encoding, and the remaining platform-semantic work. Increment 8a
admits the closed fundamental `Type` identities without runtime reflection;
8a1 adds closed named Constraint-object metadata and private observation tags;
8a2 applies closed Int constraints, retains static evidence over unchanged base
storage, and reuses the existing Result/Error path for dynamic validation; and
8a3 retains closed Type identity/view and initial language-context metadata only
through checking, folds exact static relations, and materializes the ordinary
numeric Version without a reflection runtime; and 8a4 retains the six closed
atomic Capability identities and canonical composition only in checked compiler
metadata, emitting at most a final constant textual observation; and 8a5
retains one direct source-root function interface and its exact role-to-function
evidence through checking, then emits only the ordinary direct functions and
truthful runtime debug information. 8b retains
open-world type and introspection metadata, parameterized capability claims and
operation evidence, general constraint evidence/application, later context
changes, and the remaining assurance work.
