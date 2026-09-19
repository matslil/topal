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

## TOPAL-COMP-DIAGNOSTIC-CONTROL-001 — Static diagnostic controls

The compiler shall accept legacy warning controls and structured
diagnostic-identity controls whose stack and next-statement structure the shared
parser has validated. Shared underflow, mismatch, missing-target, and unclosed-
stack diagnostics shall reach the compiler caller unchanged. Language errors
shall remain unsuppressible.

The checked compiler model shall erase valid controls after their static effect
has been accounted for. Because this increment emits no configurable compiler
warnings or severity-neutral diagnostics, it has no diagnostic event to filter;
future such diagnostics shall consult the active source identity and lexical
extent. LLVM IR and the executable shall contain no operation or state for an
erased control. Native tests shall cover both control spellings, shared invalid-
stack validation, exact interpreter/compiler output, absent runtime lowering,
freestanding artifacts, source debugging, the shared corpus, and separate
resource baselines.

This shall add no runtime diagnostic table, allocation, foreign dependency,
C/C++ runtime, other-language standard library, public ABI, or
`topal-native/6` revision. This realizes
`TOPAL-COMPILER-DIAGNOSTIC-CONTROL-001` and `TOPAL-SYN-DIAG-001` for compiler
increment 1a.

## TOPAL-COMP-LINT-VARIANT-001 — Static lint language context

The compiler shall accept a v0.1 source context whose only canonical optional
feature is `lint`, preserve that feature in the checked program and in any
admitted static `lang context` value, and expose `lang lint` only in that
context. The exposed value shall be the empty, authority-free `lang lint`
namespace with classifier `Scope`; without the feature, the shared
`E-LINT-VARIANT` diagnostic shall be retained. Every other optional feature
shall remain rejected with `E-COMPILER-UNSUPPORTED`.

Linux x86-64 lowering may extend the private `Scope` enumeration with a stable
compiler-selected tag for canonical display and truthful DWARF/GDB inspection.
It shall add no runtime namespace or lint table, lookup, allocation, filesystem,
network, process, debugger, or application authority, foreign dependency,
C/C++ runtime, other-language standard library, public ABI, or
`topal-native/6` revision.

A future compiled-library interface shall identify the selected language
revision, canonical variant feature set, exposed vocabulary, and authority
profile independently of this private tag and native symbol spelling. Tests
shall compile the unchanged interpreter regression, compare exact output,
inspect checked context metadata and O0 LLVM, validate the shared corpus and
separate resource baselines, and verify freestanding ELF, DWARF, GDB, undefined
symbols, needed libraries, and relocations. This realizes
`TOPAL-COMPILER-LINT-VARIANT-001`, `TOPAL-SYN-CONTEXT-001`, and
`TOPAL-LINT-VARIANT-001` for compiler increment 1b.

## TOPAL-COMP-INT-001 — Arbitrary finite Int runtime

The compiler shall remove the increment-1 signed-64-bit admission boundary and
represent every admitted finite `Int` with dynamically sized exact storage. At
O0, literals, negation, absolute value, addition, subtraction, multiplication,
equality, ordering, function passage, decision joins, decimal output, and GDB
inspection shall preserve the normative value for operands of any size that
available address space can hold.

The private `topal-native/6` representation shall be canonical, immutable, and
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

## TOPAL-COMP-NAT-COMPARISON-001 — Nat comparison evidence

The compiler shall evaluate each admitted Nat comparison operand once and
forget its validated constraint evidence to the unchanged canonical
arbitrary-precision Int value. Equality, inequality, ordered predicates, and
three-way comparison between Nat values or mixed Nat/Int values shall use the
existing Int comparison. Mixed Nat/Rational comparison shall use the existing
canonical Int-to-Rational conversion and Rational comparison.

Evidence forgetting shall emit no runtime conversion, unsigned representation,
fixed-width narrowing, allocation, Nat-specific operation, or ABI change. The
original classified binding shall retain Nat DWARF identity. Nat shall also be
admitted recursively as same-classifier positional-product equality evidence.
This requirement covers `TOPAL-NUM-NAT-001`, `TOPAL-TYPE-CONSTRAINT-001`,
`TOPAL-TYPE-EQUALITY-001`, `TOPAL-TYPE-ORDERING-001`,
`TOPAL-NUM-COMPARE-001`, and `TOPAL-NUM-THREE-WAY-COMPARE-001`; it realizes
`TOPAL-COMPILER-NAT-COMPARISON-001` for compiler increment 2e.

## TOPAL-COMP-MODULAR-001 — Nominal modular numbers

The compiler shall collect admitted root-scope `ModNat` and `ModInt`
declarations whose modulus is given by a direct finite inclusive Int range,
retain their nominal identity and canonical bounds, and reject malformed or
cross-nominal operations. Checked construction shall require statically proven
in-range input until dynamic Result-producing construction is available.
Explicit `value modulo Type` shall accept an admitted Int value and reduce it
to the unique representative.

Wrapping addition, subtraction, multiplication, and negation shall use the
existing exact arbitrary-precision Int runtime before canonical reduction.
Equality, ordering, and three-way comparison shall compare canonical
representatives only for identical modular types. Canonical display shall
include the source type name. Private function passage, returns, and decision
joins shall retain the classifier while reusing the existing Int pointer
carrier; LLVM shall own its physical x86-64 call lowering.

DWARF shall give every modular declaration a distinct semantic typedef and
storage identity, and the bundled GDB renderer shall safely show both the type
name and canonical value. Tests shall compile the unchanged interpreter
modular-number regression, compare exact output, inspect checked and LLVM
lowering, reject invalid construction and nominal mixing, and verify undefined
symbols, needed libraries, relocations, and GDB parameter values.

This increment shall use only the existing Topal Linux syscall allocator and
writer. It adds no machine-width wraparound, LLVM-optimization dependency,
foreign allocator, C/C++ runtime, other-language standard library, public ABI,
or `topal-native/6` revision. Named range operands, dynamic checked
construction, modular absolute value, persistent/public representation,
serialization, and compiled-library metadata remain deferred. This realizes
`TOPAL-COMPILER-MODULAR-001` and the admitted portions of
`TOPAL-NUM-MODULAR-TYPE-001`, `TOPAL-NUM-MODULAR-CONSTRUCT-001`,
`TOPAL-NUM-MODULAR-REDUCE-001`, and `TOPAL-NUM-MODULAR-ARITHMETIC-001` for
compiler increment 2f.

## TOPAL-COMP-MODULAR-CONSTRUCTION-001 — Named ranges and dynamic construction

The compiler shall accept a root modular declaration whose operand names an
earlier root binding initialized by a closed finite inclusive `Range Int`.
Resolution shall follow immutable source order and preserve the same exact
canonical bounds as direct range syntax; dynamic, cyclic, forward, malformed,
and non-range operands shall remain rejected.

For checked `Name value`, statically proven in-range inputs shall retain the
direct nominal value path and closed proven out-of-range inputs shall retain
the source diagnostic. Every other admitted Int shall be evaluated once and
compared against both arbitrary-precision inclusive bounds in generated LLVM
control flow. Success shall wrap the original Int pointer; rejection shall
produce the existing arithmetic Result/Error representation with code
`out-of-range`, lexical domain `root.Name(Int)`, and exact operand provenance.

`Result (Name, lang arithmetic ArithmeticErrorCode)` shall be admitted through
the existing private pointer call, return, projection, decision, display, and
Error-observation paths. Its DWARF header shall retain a nominally typed Name
payload so the bundled GDB renderer can validate and print success and failure
values. Tests shall use one unchanged interpreter/compiler regression, compare
exact output, inspect checked and LLVM control flow, and verify no undefined
symbol, needed library, dynamic relocation, or lost GDB type/value/frame.

This increment shall reuse only the existing exact Int comparison, Result,
Error, Linux syscall allocation, and writer facilities. It adds no C/C++
runtime, other-language standard library, public ABI, serialization contract,
or `topal-native/6` revision. Modular absolute value, persistent/public
representation, serialization, introspection, and compiled-library metadata
remain deferred. This realizes
`TOPAL-COMPILER-MODULAR-CONSTRUCTION-001` and completes the named-range and
dynamic-construction portions of `TOPAL-NUM-MODULAR-TYPE-001` and
`TOPAL-NUM-MODULAR-CONSTRUCT-001` for compiler increment 2f1.

## TOPAL-COMP-RESULT-001 — Dynamic arithmetic Results

The compiler shall represent an admitted arithmetic `Result` with an immutable
success-or-error tag and one statically classified payload. It shall represent
every intrinsic Error with its reporting domain, nominal arithmetic code,
absent detail and cause, and source file, line, and column provenance. The
private representation shall be versioned as `topal-native/6`, remain opaque to
foreign calling conventions, and be inspectable through the bundled GDB
renderer.

Dynamic finite Rational construction shall distinguish `division-by-zero` from
`indeterminate`. Dynamic Rational division and negative power, plus dynamic Int
modulo and quotient/modulo, shall return `division-by-zero` from their specified
reporting domains rather than terminate the process. Ordinary success values
shall satisfy explicit Result contracts, and returning or passing through a
failed Result shall preserve the complete Error unchanged.

This requirement covers `TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001`,
`TOPAL-NUM-INT-MODULO-001`, `TOPAL-NUM-INT-QUOTIENT-MODULO-001`,
`TOPAL-NUM-DYNAMIC-DIVZERO-001`, `TOPAL-NUM-RAT-NEG-POW-001`,
`TOPAL-NUM-ARITHMETIC-ERROR-001`, and the admitted function-contract and
propagation cases of `TOPAL-TYPE-RESULT-001`. It realizes
`TOPAL-COMPILER-RESULT-001` for compiler increment 2c-b1.

Exact checked Int construction and an Int-classified binding shall validate a
dynamically obtained Rational denominator, returning `not-representable` from
`root.Int(Rational)` without rounding or truncation. Checked Nat construction
shall preserve a nonnegative Int or return `out-of-range` from `root.Nat(Int)`.
A compatible classified binding shall project the success payload and return
the original complete Error immediately on failure.

These additions cover `TOPAL-NUM-RATIONAL-INT-EXACT-001`,
`TOPAL-NUM-RATIONAL-INT-VALIDATE-001`, `TOPAL-NUM-INT-CONSTRUCT-001`,
`TOPAL-NUM-NAT-CONSTRUCT-001`, and `TOPAL-TYPE-RESULT-PROJECT-001`. They realize
the remaining `TOPAL-COMPILER-RESULT-001` scope for compiler increment 2c-b2.

Result decisions shall evaluate their subject once, bind success or whole-Error
payloads only in the selected action, and execute only that action. Qualified
arithmetic Error-code matchers shall compare the stored nominal code rather
than the independent reporting domain, preserve source ordering, diagnose
unknown or duplicate codes, and require either a generic Error fallback or the
complete four-code vocabulary. Compatible String-valued actions shall use an
immutable native descriptor and merge without eager evaluation.

Selecting `code` or `domain` from an arithmetic Error shall return the stored
typed value without reconstructing or altering the Error. The compiler shall
represent literal String bytes, immutable descriptors with cached canonical
display spellings, Error fields, functions, decision joins, textual output,
DWARF, and GDB consistently under `topal-native/6`. Pointer-bearing descriptors
shall be constructed at run time so static no-loader PIEs retain no load-time
relocations. String display shall produce the canonical ordinary or
conflict-free tagged literal form using only the Topal Linux syscall runtime.

These additions cover `TOPAL-DECISION-RESULT-001`,
`TOPAL-DECISION-ERROR-CODE-001`, `TOPAL-ERROR-FIELD-001`, and the admitted
literal transport and display cases of `TOPAL-SYN-STRING-001`. They complete
compiler increment 2c-b3; generic Result/library composition and the remaining
String operations retain their later roadmap dispositions.

## TOPAL-COMP-ERROR-OPTIONAL-FIELDS-001 — Optional Error provenance fields

The compiler shall classify `detail`, `cause`, and `source` selection from an
admitted Error or failed Result as `Optional String`, `Optional Error`, and
`Optional SourceLocation`. It shall wrap the existing nullable detail and cause
slots without synthesizing information. A present source descriptor shall
produce a private immutable SourceLocation containing canonical Int pointers
for the stored one-based line and column; an absent descriptor shall produce
`None`. Selection shall not reconstruct or alter the Error.

The existing private Optional header shall carry all three results across
admitted decisions and private function boundaries. LLVM shall own physical
x86-64 call lowering while the frontend retains the payload classifier.
Canonical output shall render a present source as
`Some (line is N, column is N)`. DWARF shall describe SourceLocation and both
new Optional payload types, and the bundled GDB renderer shall safely render
them. Tests shall compile the unchanged interpreter composition regression,
compare its exact output, inspect the lowering and DWARF, and verify undefined
symbols, needed libraries, relocations, and GDB values.

SourceLocation and Optional allocations shall use only the existing Topal
Linux `mmap` boundary. The already reserved Error layout and private opaque-
pointer Optional layout remain `topal-native/6`; this increment adds no foreign
allocator, C/C++ runtime, other-language standard library, public ABI, or ABI
revision. This realizes `TOPAL-COMPILER-ERROR-OPTIONAL-FIELDS-001` and the
remaining field-selection portion of `TOPAL-ERROR-FIELD-001` for compiler
increment 2c-b5.

## TOPAL-COMP-CODE-001 — Qualified arithmetic ErrorCode values

The compiler shall resolve the four qualified values published by
`lang arithmetic` to the same closed nominal `ArithmeticErrorCode` identity and
sealed `i32` tags stored in compiled Error values. Direct values shall support
same-type equality, scalar function passage, canonical alternative-label
display through the Topal platform writer, DWARF enumerators, and GDB
inspection without a runtime namespace lookup or foreign-language dependency.
An unknown or incompletely qualified code shall not be reinterpreted as an
arithmetic code.

This requirement covers `TOPAL-NUM-ARITHMETIC-ERROR-001` and realizes
`TOPAL-COMPILER-ERROR-CODE-001` for compiler increment 2c-b4.

## TOPAL-COMP-GENERATOR-ERROR-CODE-001 — Qualified generator code value

The checked compiler shall resolve only `lang generator generator-closed` as
the sole initial alternative of the nominal
`lang generator GeneratorErrorCode` enum. It shall retain the exact nominal
identity through immutable bindings, same-type equality, decomposed products,
canonical output, DWARF, and GDB. Unknown, incomplete, and arithmetic-code names
shall not be accepted as this vocabulary.

The Linux x86-64 backend may reuse its private enum lowering with tag zero.
Constructing or observing the value shall not allocate or control a generator,
deliver a close signal, choose an `Error.domain`, or fabricate generator/yield
provenance. The tag shall remain compiler-private rather than a public,
foreign, persistent, serialized, or compiled-library identity; future library
metadata shall carry canonical vocabulary and alternative identities instead.
This increment shall admit no Generator classifier or function boundary,
generator state/storage, suspension/resumption, close delivery/handling, or
Error carrier. It shall require no generator runtime, allocator, foreign
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, or `topal-native/6` revision. This realizes
`TOPAL-COMPILER-GENERATOR-ERROR-CODE-001` and
`TOPAL-GENERATOR-ERROR-CODE-001` for compiler increment 5a.

## TOPAL-COMP-GENERATOR-ITERATE-CONSTRUCT-001 — Lazy iterate construction

The checked compiler shall admit exact `Generator Int Unit Unit` construction
from an `Int` initial expression and unary anonymous `Int -> Int` next
operation, followed optionally and directly by `take-while` with a unary
anonymous `Int -> Boolean` predicate. It shall evaluate the initial expression
once, retain the checked construction and anonymous bodies, invoke neither body
during construction, and print the canonical `<Generator Int Unit Unit>` final
observation.

An immutable local Generator binding shall be consumable at most once. Before
LLVM lowering, the checked boundary shall reject repeated, abandoned, or
explicitly discarded bindings; product containment; qualified access;
equality; decision joins; and function or library boundaries. The Linux x86-64
backend may lower the remaining construction-only binding to a private `i32`
observation token with semantic Generator DWARF, but that token shall encode no
seed, operation, capture, continuation, ownership, or stable identity. Future
compiled-library metadata shall describe the canonical Generator classifier,
captured operations, and evidence independently of target representation.

This increment shall perform no traversal, yield, resume, suspension, close
delivery, generator allocation, indirect call, or generator runtime operation.
It shall introduce no foreign dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, or `topal-native/6`
revision. This realizes
`TOPAL-COMPILER-GENERATOR-ITERATE-CONSTRUCT-001`,
`TOPAL-GENERATOR-ITERATE-001`, and `TOPAL-GENERATOR-TAKE-WHILE-001` for compiler
increment 5b.

## TOPAL-COMP-GENERATOR-UNFOLD-CONSTRUCT-001 — Lazy unfold construction

The checked compiler shall admit exact `Generator Int Unit Unit` construction
from a `List Int` seed expression and a unary anonymous
`List Int -> Optional (Int, List Int)` step operation. It shall preserve the
distinction between seed and yield classifiers, evaluate the seed once, retain
the checked seed and anonymous operation, invoke no step during construction,
and print the canonical `<Generator Int Unit Unit>` final observation.

The existing one-consumption local Generator boundary shall reject repeated,
abandoned, or explicitly discarded bindings; product containment; qualified
access; equality; decision joins; and function or library boundaries. A step
that captures another Generator shall also be rejected before LLVM. The Linux
x86-64 backend may reuse the private `i32` construction observation token with
semantic Generator DWARF. That token shall encode no seed, step, capture,
continuation, ownership, or stable identity. Future compiled-library metadata
shall carry the canonical Generator classifier, distinct seed and yield
classifiers, step signature, captured operations, and evidence independently
of a target representation adapter.

This increment shall perform no traversal, collection, yield, resume,
suspension, close delivery, Generator allocation, indirect call, or Generator
runtime operation. It shall introduce no foreign dependency, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library Generator ABI, or `topal-native/6` revision. This realizes
`TOPAL-COMPILER-GENERATOR-UNFOLD-CONSTRUCT-001` and
`TOPAL-GENERATOR-UNFOLD-001` for compiler increment 5d.

## TOPAL-COMP-GENERATOR-UNFOLD-COLLECT-001 — Finite unfold collection

The checked compiler shall admit unary `collect` over an exact
`Generator Int Unit Unit` whose retained construction uses an immutable
`List Int` seed binding and exactly `uncons` of its unary `List Int` parameter
as the step. The Generator may be consumed directly or after linear local
moves. The seed shall have been evaluated once at construction. Collection
shall append every nonempty seed head to a fresh `List Int` in order, continue
with the corresponding tail, terminate without an entry at `Empty`, and leave
the source List unchanged.

An arbitrary step, step statements or captures, a seed whose evaluated
identity is unavailable to the specialization, and a Generator without local
construction provenance shall be rejected before LLVM. Provenance retained for
this proof shall remain compiler-private and shall not enter persistent,
serialized, public, foreign, or compiled-library metadata.

The Linux x86-64 backend shall use explicit current-seed and result-head/tail
LLVM SSA loop values and may allocate fresh immutable List nodes through the
Topal platform allocator. This required O0 semantic lowering shall materialize
no Optional, call no List `uncons` helper, use no construction token as state,
allocate no Generator object, and emit no callback, indirect call, or generic
Generator runtime. It shall introduce no foreign dependency, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library Generator ABI, or `topal-native/6` revision. Future library
metadata shall identify the canonical Generator, seed/yield classifiers, exact
step/evidence, linear ownership, and target adapter independently of this
specialization. This realizes
`TOPAL-COMPILER-GENERATOR-UNFOLD-COLLECT-001`,
`TOPAL-GENERATOR-UNFOLD-001`, and `TOPAL-GENERATOR-UNFOLD-COLLECT-001` for
compiler increment 5e.

## TOPAL-COMP-GENERATOR-ITERATE-COLLECT-001 — Finite iterate collection

The checked compiler shall admit unary `collect` only for a syntactically
direct exact `Generator Int Unit Unit` formed by `iterate` and bounded by
`take-while`. It shall evaluate the initial expression once, test each
candidate once before publication, append every accepted Int to a fresh
`List Int` in yield order, and invoke the next operation once afterward. A
rejected candidate shall neither enter the List nor cause another next
invocation. The resulting value shall use the already admitted List behavior
for equality, display, private passage, DWARF, and GDB.

An unbounded iterate, an indirect or stored Generator operand, and an anonymous
next or predicate body with an outer-value capture shall be rejected before
LLVM. The Linux x86-64 backend shall carry only the current arbitrary-precision
Int and private List head/tail as explicit SSA loop state, allocate immutable
List nodes through the Topal platform allocator, and use direct generated
control flow. It shall allocate no Generator object, use no construction token
as executable state, host recursion, indirect call, or generic Generator
runtime, and introduce no foreign dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library Generator
ABI, or `topal-native/6` revision. Future compiled-library metadata shall
describe the canonical operation/evidence composition independently of this
specialization. This realizes
`TOPAL-COMPILER-GENERATOR-ITERATE-COLLECT-001`,
`TOPAL-GENERATOR-ITERATE-001`, `TOPAL-GENERATOR-TAKE-WHILE-001`, and
`TOPAL-GENERATOR-COLLECT-001` for compiler increment 5c.

## TOPAL-COMP-GENERATOR-ITERATE-FOREACH-001 — Bounded iterate traversal

The checked compiler shall admit a root `foreach` statement that consumes an
exact `Generator Int Unit Unit` retained from an Int-literal `iterate` with
capture-free unary `Int -> Int` next and `Int -> Boolean` `take-while`
operations. Its capture-free body shall bind the accepted Int and produce Unit.
The statement may bind its Unit result, optionally classified as Unit, or leave
that result unnamed.

Every candidate shall be tested once before visitation. Each accepted candidate
shall execute the body once and only then invoke next once. The first rejected
candidate shall execute neither body nor next, and traversal shall return Unit.
The source Generator shall be consumed through the existing local linearity
boundary. Dynamic initial values, unbounded or differently constructed
Generators, and captures in next, predicate, or body shall be rejected before
LLVM.

The Linux x86-64 backend shall carry the current arbitrary-precision Int as
explicit LLVM SSA loop state and emit direct predicate, body, and next blocks.
This required O0 semantic lowering shall expose the current body binding and
Unit result through DWARF/GDB, allocate no traversal collection or Generator
object, use no construction token as state, host recursion, callback, indirect
call, or generic Generator runtime, and introduce no foreign dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library Generator ABI, or `topal-native/6` revision. Future
library metadata shall identify canonical Generator, operation/predicate/body,
evidence, capture-layout, ownership, and target-adapter identities rather than
this specialization. This realizes
`TOPAL-COMPILER-GENERATOR-ITERATE-FOREACH-001`,
`TOPAL-GENERATOR-ITERATE-001`, `TOPAL-GENERATOR-TAKE-WHILE-001`, and
`TOPAL-GENERATOR-ITERATE-FOREACH-001` for compiler increment 5f.

## TOPAL-COMP-GENERATOR-SINGLE-YIELD-001 — Closed custom suspension

The checked compiler shall admit one root custom generator declaration with
one ordinary Character input, Character yield, Unit resume, and Unit final
result when its body is exactly one discarded yield of that input followed by
Unit. Applying the declaration shall evaluate and classify the initial operand
once, start a fresh linear Generator, retain the declaration identity and exact
yield as compile-session provenance, and expose the first suspension before
later source execution. The admitted exact body has no later computation.

Root foreach shall consume one locally bound instance, invoke its capture-free
Character-to-Unit action exactly once with the yielded Character, resume with
Unit, and return the distinct final Unit. The source shall use the existing
one-consumption local Generator boundary; repeated use and abandonment other
than the exact function-scope close admitted by
`TOPAL-COMP-GENERATOR-CLOSE-001` shall remain rejected. Declaration shapes not
covered by a later compiler rule, overloads, dynamic Character provenance,
direct traversal without a retained local, non-Unit actions, captures, and
function transfer except for the exact result admitted by
`TOPAL-COMP-CUSTOM-GENERATOR-RESULT-001` or library transfer boundaries shall be
rejected before LLVM.

The Linux x86-64 backend shall evaluate the initial Character once, lower the
proven single suspension and resumption as ordered inline code, and use the
compiler-private Generator observation token only for ownership and debugging,
not continuation state. This required O0 source-semantic lowering shall expose
the Generator and yielded Character bindings through DWARF/GDB. It shall add no
Generator object, state allocation, dispatcher, callback, indirect call,
unwind support, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library Generator ABI, or `topal-native/6` revision.
Future compiled-library metadata shall carry the canonical declaration,
Generator directions, suspension points, capture/effect evidence, ownership,
close behavior, and target-adapter identities rather than this specialization.
This realizes `TOPAL-COMPILER-GENERATOR-SINGLE-YIELD-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-GENERATOR-FOREACH-001` for compiler increment 5g.

## TOPAL-COMP-GENERATOR-MULTIPLE-YIELD-001 — Repeated custom suspensions

The checked compiler shall generalize the admitted root Character generator to
one or more consecutive discarded yields of its sole initial Character followed
by final Unit. It shall retain the exact source-ordered yield sequence as
compile-session provenance. Application shall still evaluate the initial
operand exactly once and stop at the first suspension; each successful Unit
resumption shall advance to the next retained yield or the final Unit.

Root foreach shall consume one locally bound instance and invoke its
capture-free Character-to-Unit action exactly once for each yielded value in
source order. It shall resume with Unit after every action, including the final
action before returning Unit. Existing repeated-use and abandonment rejection
shall continue to apply. Exact pre-yield Unit completion is governed by
`TOPAL-COMP-GENERATOR-EARLY-RETURN-001`; ordinary statements between yields
other than the exact local activation admitted by
`TOPAL-COMP-GENERATOR-SUSPENSION-001`, different yield expressions, overloads,
dynamic Character provenance, captures, close handling, and function or
library boundaries shall remain rejected before LLVM.

The Linux x86-64 backend shall expand the finite proven sequence as ordered
inline action blocks with an erased Unit resumption between adjacent blocks.
This is mandatory O0 semantic lowering, not loop unrolling delegated to LLVM.
The existing compiler-private Generator observation token and debug-only
Character shadow may be reused, but no token shall act as continuation state.
Generated code shall add no Generator object or state allocation, dispatcher,
callback, indirect call, unwind support, C/C++ runtime, other-language standard
library, needed library, dynamic relocation, public/library Generator ABI, or
`topal-native/6` revision. Future compiled-library metadata shall describe the
ordered suspension graph and its canonical declaration/direction, effect,
ownership, close, and target-adapter evidence. This realizes
`TOPAL-COMPILER-GENERATOR-MULTIPLE-YIELD-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-GENERATOR-FOREACH-001` for compiler increment 5h.

## TOPAL-COMP-GENERATOR-LOCAL-BINDING-001 — Exact local Character state

The checked compiler shall admit the existing custom Character generator when
its body begins with exactly one explicitly classified, immutable Character
binding whose value is the sole initial Character parameter, followed by one
or more consecutive discarded yields of that local and final Unit. Application
shall evaluate the initial operand once, evaluate the alias binding in the
generator scope before the first suspension, and retain the local identity and
exact Character value as compile-session provenance. The local shall remain
available to every retained yield and shall never enter the caller's checked
environment.

Root foreach shall preserve the existing linear consumption and ordered
action/Unit-resumption behavior. The Linux x86-64 backend shall materialize the
proven immutable Character value and expose a debug-only pointer shadow for the
local in a generator lexical DWARF scope before the first action. That shadow
may persist across the admitted yields but shall be out of scope after the
traversal. This mandatory O0 lowering shall not depend on optimization or
introduce a semantic continuation object, state allocation, dispatcher,
callback, indirect call, unwind support, C/C++ runtime, other-language standard
library, needed library, dynamic relocation, public/library Generator ABI, or
`topal-native/6` revision.

Unclassified or differently classified locals, non-identity initializers,
additional statements not admitted by `TOPAL-COMP-GENERATOR-SUSPENSION-001`,
dynamic Character provenance, captures, resume bindings other than the exact
Unit success binding admitted by `TOPAL-COMP-GENERATOR-RESUME-BINDING-001`,
close handling, and function or library boundaries shall remain rejected
before LLVM. Future compiled-library metadata shall encode
canonical local-state identities and types with the declaration, suspension
graph, directions, captures/effects, ownership/close behavior, and target
adapters rather than serialize this executable-local debug shadow. This
realizes `TOPAL-COMPILER-GENERATOR-LOCAL-BINDING-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-LOCAL-BINDING-001`,
`TOPAL-GENERATOR-SUSPEND-001`, and `TOPAL-GENERATOR-FOREACH-001` for compiler
increment 5i.

## TOPAL-COMP-GENERATOR-EARLY-RETURN-001 — Completion before suspension

The checked compiler shall admit the existing root custom Character generator
directions when the complete body is exactly final Unit and therefore reaches
its declared Unit result before any yield. Application shall evaluate and
classify the initial Character exactly once, create a fresh linear Generator,
and retain an empty suspension sequence plus completed Unit as compile-session
provenance. The absence of a yield shall not skip static checking of the
Character-to-Unit foreach action.

Root foreach shall consume one locally bound admitted instance, invoke the
action zero times, and produce the generator's final Unit directly. Repeated
use and abandonment shall remain rejected under the existing local linearity
boundary. Additional body statements, generator locals, explicit return, other
input/yield/resume/result classifiers, non-Unit final values, overloads,
captures, close handling, and function or library boundaries shall remain
rejected before LLVM unless another compiler rule admits them.

On Linux x86-64, the backend shall evaluate the initial Character once and
lower the empty suspension sequence and final Unit with no action block or
action-parameter debug storage. The compiler-private Generator observation
token may remain for ownership and GDB identity, but shall not represent a
continuation state. This mandatory O0 lowering shall not depend on dead-code
elimination or introduce a Generator object, state allocation, dispatcher,
callback, indirect call, unwind support, C/C++ runtime, other-language standard
library, needed library, dynamic relocation, public/library Generator ABI, or
`topal-native/6` revision. Future compiled-library metadata shall encode the
terminal-before-suspension graph, final value, directions, ownership/close
behavior, and target adapters canonically. This realizes
`TOPAL-COMPILER-GENERATOR-EARLY-RETURN-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-EARLY-RETURN-001`, and
`TOPAL-GENERATOR-FOREACH-001` for compiler increment 5j.

## TOPAL-COMP-GENERATOR-FINAL-CHARACTER-001 — Distinct final Character

The checked compiler shall admit one root custom generator with a Character
input, Character yield, Unit resume, and Character final result when its body
is exactly one discarded yield of the initial parameter followed by an exact
closed Character literal. Application shall evaluate the initial Character
once, retain the yielded value and separate final value as compile-session
provenance, and stop at the yield without evaluating the final expression.

Direct root foreach over one locally bound admitted instance shall consume the
Generator, invoke its checked Character-to-Unit action exactly once with the
yielded Character, resume with Unit, then evaluate and produce the distinct
final Character. The final expression shall not be emitted before the action
and resumption. Existing repeated-use and abandonment rejection shall continue
to apply. Binding the non-Unit foreach result, zero or multiple yields, a local
alias, dynamic or non-Character final expressions, other directions, overloads,
captures, close handling, and function or library boundaries shall remain
rejected before LLVM.

On Linux x86-64, the backend shall lower the yield/action/resume sequence and
final Character materialization in that explicit order at O0. It may retain
the compiler-private Generator observation token and existing debug-only
yielded-Character shadow, but neither shall carry the final value or act as
continuation state. Generated DWARF shall expose the full
`Generator Character Unit Character` direction identity and yielded Character
to GDB. The lowering shall introduce no Generator object, state allocation,
dispatcher, callback, indirect call, unwind support, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library Generator ABI, or `topal-native/6` revision. Future
compiled-library metadata shall encode the result classifier and final-value
node alongside canonical declaration, direction, suspension, effect,
ownership/close, and target-adapter evidence. This realizes
`TOPAL-COMPILER-GENERATOR-FINAL-CHARACTER-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-FINAL-RETURN-001`,
`TOPAL-GENERATOR-SUSPEND-001`, and `TOPAL-GENERATOR-FOREACH-001` for compiler
increment 5k.

## TOPAL-COMP-GENERATOR-SUSPENSION-001 — Post-resume local activation

The checked compiler shall admit the Unit-final custom Character-generator
subset when its body contains one or more discarded yields of the sole initial
Character, then exactly one explicitly classified immutable Character binding
initialized from that initial parameter, one or more discarded yields of the
local, and final Unit. It shall retain the local identity, classifier, source
span, exact Character value, and the number of successful resumptions before
activation. Starting the generator shall stop at its first yield and shall not
evaluate the post-yield binding.

Direct root foreach over one locally bound admitted instance shall consume the
Generator, complete each prefix action and Unit resumption, activate the local
only after the last prefix resumption, then observe the local yields in source
order before final Unit. The local shall remain absent from the caller's
checked environment. A local without a later yield, a second local, a
non-identity initializer, a yield of the wrong active value, other ordinary
statements, dynamic Character provenance, captures, resume bindings other than
the exact Unit success binding admitted by
`TOPAL-COMP-GENERATOR-RESUME-BINDING-001`, close handling, and function or
library boundaries shall remain rejected before LLVM.

On Linux x86-64, the backend shall emit the completed prefix action and erased
Unit resumption before local materialization and its lexical DWARF declaration,
then emit that declaration before the next yield action at O0. The checked
resumption count shall guide source-order expansion but shall not become a
runtime program counter or public layout. This lowering shall not depend on
optimization or introduce a Generator object, semantic continuation state,
dispatcher, callback, indirect call, unwind support, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library Generator ABI, or `topal-native/6` revision. Future
compiled-library metadata shall identify each local activation transition
canonically with the declaration, directions, ordered suspension graph,
captures/effects, ownership/close behavior, and target adapters. This realizes
`TOPAL-COMPILER-GENERATOR-SUSPENSION-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-BODY-STATEMENT-001`,
`TOPAL-GENERATOR-LOCAL-BINDING-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-GENERATOR-FOREACH-001` for compiler increment 5l.

## TOPAL-COMP-GENERATOR-RESUME-BINDING-001 — Exact Unit resumption

The checked compiler shall admit the root custom
`Generator Character Unit Unit` subset when its body is exactly one named,
optionally Unit-classified binding of `yield` applied to the sole initial
Character, followed by that binding as the final Unit expression. It shall
retain the yielded Character and a distinct Unit local whose activation follows
one successful resumption. Starting the generator shall stop at the yield
without introducing or evaluating the resume binding.

Direct root foreach over one locally bound admitted instance shall consume the
Generator, invoke its Character-to-Unit action once, resume with Unit, make
that Unit available under the generator-local name, and only then use it as the
final result. The name shall remain absent from the caller's checked
environment. A wrong or discarded binding name, non-Unit classifier, different
yielded value, additional yield or body statement, different final expression,
dynamic Character provenance, captures, close handling, and function or
library boundaries shall remain rejected before LLVM.

On Linux x86-64, the backend shall emit the action before a debug-only `i8`
shadow for the successfully resumed Unit and expose that binding as Unit to GDB
at its source line at O0. The erased success value shall not create semantic
continuation state or conflate ordinary Unit resumption with the distinct
`generator-closed` close edge. The lowering shall not depend on optimization or
introduce a Generator object, state machine, dispatcher, callback, indirect
call, unwind support, C/C++ runtime, other-language standard library, needed
library, dynamic relocation, public/library Generator ABI, or `topal-native/6`
revision. Future compiled-library metadata shall encode success and close
edges, resume-local activation, the declaration and directions, suspension
identity, captures/effects, ownership/close behavior, and target adapters
canonically. This realizes `TOPAL-COMPILER-GENERATOR-RESUME-BINDING-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`,
`TOPAL-GENERATOR-RESUME-BINDING-001`, and `TOPAL-GENERATOR-FOREACH-001` for
compiler increment 5m.

## TOPAL-COMP-GENERATOR-CLOSE-001 — Exact function-scope abandonment

The checked compiler shall admit one call-specialized ordinary function whose
body binds one fresh instance of the exact single-yield custom
`Generator Character Unit Unit` and then reaches final Unit without consuming
it. The initial Character shall retain its exact caller provenance. Function
scope exit shall consume the suspended Generator, deliver `generator-closed`
with lexical domain `root`, retain the root generator declaration as separate
provenance, and complete the handler-free generator boundary before returning
Unit.

The checked function body shall contain an explicit custom close after the
Generator binding and before its final expression. Root abandonment, multiple
owned generators, multiple yields, generator locals, and close handlers or
other post-yield work except for the exact handler admitted by
`TOPAL-COMP-GENERATOR-CLOSE-HANDLER-001`, dynamic Character provenance without
call specialization, static or anonymous functions, explicit return, non-Unit
final results, Generator parameter/result transfer, and library boundaries
shall remain rejected before LLVM.

On Linux x86-64, the backend may erase the intrinsic close value and final Unit
for this handler-free, effect-free body after preserving their checked O0
order; the expected close signal has no source observer at the generator
boundary. The compiler-private ownership/debug token shall be consumed
conceptually and remain visible as `Generator Character Unit Unit` in the
function's DWARF scope. The lowering shall not introduce a Generator object,
continuation state, close dispatcher, callback, indirect call, unwind support,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library Generator ABI, or `topal-native/6` revision. Future
compiled-library metadata shall encode the close site and lexical domain,
generator declaration provenance, close/success edges, suspension identity,
cleanup/effect evidence, ownership state, and target adapters canonically.
This realizes `TOPAL-COMPILER-GENERATOR-CLOSE-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-CLOSE-001`, and
`TOPAL-GENERATOR-ERROR-CODE-001` for compiler increment 5n.

## TOPAL-COMP-GENERATOR-CLOSE-HANDLER-001 — Exact close-result handling

The checked compiler shall generalize the admitted function-local custom close
to one root generator declaration whose body binds its sole Character yield
result and immediately selects a complete `Error`/`Ok` decision with Unit
actions. The checked generator construction shall retain the yield-result
binding, Error and Ok binding identities and spans, both branch actions, and
the nominal `lang generator GeneratorErrorCode` set containing
`generator-closed`.

When the exact generator is abandoned by the admitted ordinary Unit function,
its checked close shall deliver
`Error(domain = root, code = generator-closed)`, select and execute only the
Error action, finish the generator boundary, and then return the function's
final Unit. The successful Unit-resume action shall remain explicit metadata but
shall not execute on this close edge. Root abandonment, successful foreach,
qualified close-code patterns except for the exact matcher admitted by
`TOPAL-COMP-GENERATOR-CLOSE-CODE-PATTERN-001`, non-Unit handler actions, handler
work beyond the exact decision, multiple yields or owners, generator locals,
dynamic provenance, transfer boundaries, and library boundaries shall remain
rejected before LLVM.

On Linux x86-64, O0 lowering shall materialize the intrinsic failure through
Topal-owned Result/Error and allocator primitives, statically select its Error
payload, and preserve source order independently of LLVM optimization. DWARF
shall expose the yield Result and selected Error with the generator Error-code
vocabulary so GDB renders `generator-closed`, not an unrelated nominal code.
The lowering shall not introduce a Generator object, continuation state, close
dispatcher, callback, indirect call, unwind dependency, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library Generator ABI, or `topal-native/6` revision. Future
compiled-library metadata shall encode handler branches and binding activation
together with the close site/domain, declaration provenance, success/close
edges, suspension, cleanup/effect evidence, ownership state, and target adapters
canonically. This realizes `TOPAL-COMPILER-GENERATOR-CLOSE-HANDLER-001`,
`TOPAL-GENERATOR-CLOSE-HANDLER-001`, `TOPAL-GENERATOR-CLOSE-001`, and
`TOPAL-GENERATOR-ERROR-CODE-001` for compiler increment 5o.

## TOPAL-COMP-GENERATOR-CLOSE-CODE-PATTERN-001 — Qualified close-code selection

The checked compiler shall generalize the admitted exact close handler to one
qualified `Error ( code is lang generator generator-closed )` Unit rule before
its generic Error fallback, together with the complete Ok Unit rule. The
checked handler shall retain the qualified rule's nominal code-set identity,
alternative, source span, and action separately from the generic Error
binding/action and Ok binding/action.

For the statically known close edge, the backend shall select and execute only
the qualified code action. Selection shall use the nominal
`lang generator GeneratorErrorCode.generator-closed` identity, not the lexical
Error domain or generator declaration provenance. The generic Error fallback
binding and Ok binding shall remain inactive. O0 lowering may resolve this known
selection directly without a runtime decision switch, but shall still
materialize the Topal-owned Result/Error failure and preserve the nominal code
observation and action source location for GDB.

Other qualified codes or vocabularies, code rules after the generic Error
fallback, missing fallbacks, multiple qualified rules, non-Unit actions,
successful traversal, dynamic close results, multiple yields or owners,
transfer boundaries, and library boundaries shall remain rejected before LLVM.
The lowering shall not add a Generator object, continuation state, dispatcher,
callback, indirect call, unwind dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library Generator
ABI, or `topal-native/6` revision. Future compiled-library metadata shall
preserve ordered nominal code matchers and actions together with fallback
bindings, success/close edges, domain and declaration provenance, suspension,
cleanup/effect evidence, ownership state, and target adapters canonically. This
realizes `TOPAL-COMPILER-GENERATOR-CLOSE-CODE-PATTERN-001`,
`TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001`,
`TOPAL-GENERATOR-CLOSE-HANDLER-001`, and `TOPAL-DECISION-ERROR-CODE-001` for
compiler increment 5p.

## TOPAL-COMP-CUSTOM-GENERATOR-RESULT-001 — Specialized custom continuation result

The checked compiler shall admit an ordinary nonrecursive called function with
exactly one named Character parameter and result classifier
`Generator Character Unit Unit` when its statement-free body returns a fresh
instance of the exact single-yield custom generator admitted by
`TOPAL-COMP-GENERATOR-SINGLE-YIELD-001`, applied directly to that parameter. The
top-level call argument shall retain one exact Character. Function exit shall
transfer the fresh suspended continuation without close delivery; the caller
shall bind and consume it exactly once through the admitted root Character
foreach.

Each call shall create a distinct private specialization. The checked program
shall retain the generator declaration, exact Character, suspension/final
graph, and ownership transfer as provenance associated with that private
symbol. The caller shall evaluate its Character argument once. The callee shall
return the compiler-private Generator token, and caller traversal shall invoke
the Character-to-Unit action once, resume with Unit, and finish with Unit using
only that call's retained provenance. Distinct calls shall not share provenance.

On Linux x86-64, LLVM `fastcc` shall choose placement for the private Character
descriptor parameter and `i32` Generator result; the backend shall hard-code no
System V register placement. A debug-only Character parameter shadow shall keep
the source value inspectable through function return. DWARF and GDB shall expose
the Character parameter, Generator result classifier/value, caller traversal
Character, and both call frames.

Static, anonymous, recursive, nested, multiple-parameter, statement-bearing,
non-parameter-derived, multiple-yield, local-state, handled-close, unbound-result,
caller-close, and library paths shall remain rejected before LLVM. The lowering
shall introduce no Generator object or state allocation, dispatcher, callback,
indirect call, unwind dependency, C/C++ runtime, other-language standard
library, needed library, dynamic relocation, public/library calling convention
or Generator ABI, or `topal-native/6` revision. Future compiled-library metadata
shall encode the canonical declaration and directions, suspension/final graph,
construction evidence, capture/effect evidence, transfer/ownership/close state,
and target adapters rather than expose the compiler-session side table or
private token. This realizes `TOPAL-COMPILER-CUSTOM-GENERATOR-RESULT-001`,
`TOPAL-GENERATOR-FUNCTION-RESULT-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-GENERATOR-FOREACH-001` for compiler increment 5q.

## TOPAL-COMP-CUSTOM-GENERATOR-PARAMETER-001 — Specialized custom continuation parameter

The checked compiler shall admit a top-level call to an ordinary nonrecursive
function with exactly one named `Generator Character Unit Unit` parameter and
Unit result when the argument is a named root binding constructed by the exact
single-yield custom generator admitted by
`TOPAL-COMP-GENERATOR-SINGLE-YIELD-001`. The function's executable body shall
consist only of one Character foreach over that parameter with a
Character-to-Unit action and final Unit. Argument evaluation shall transfer the
suspended continuation, consume the caller binding, and make the parameter its
sole owner. Successful traversal shall invoke the action once, resume with Unit,
reach final Unit, and return without close delivery.

Each call shall create a distinct private specialization. The checked program
shall map that call's retained declaration, exact Character, suspension/final
graph, and ownership edge to the parameter only while checking its body, then
restore the surrounding compiler-session provenance. Distinct calls shall not
share Characters or continuation provenance, and a caller shall not reuse the
consumed binding.

On Linux x86-64, LLVM `fastcc` shall choose placement for the private `i32`
Generator token parameter; the backend shall hard-code no System V register
placement. DWARF and GDB shall expose the Generator parameter, yielded Character,
and caller/callee frames. The token shall carry no public or semantic
continuation state.

Static, anonymous, recursive, nested, multiple-parameter, additional-body,
direct-expression-argument, multiple-yield, local-state, handled-close,
unconsumed-parameter, returned-parameter, repeated-use, and library paths shall
remain rejected before LLVM. The lowering shall introduce no Generator object
or state allocation, dispatcher, callback, indirect call, unwind dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library calling convention or Generator ABI, or
`topal-native/6` revision. Future compiled-library metadata shall encode the
canonical declaration and directions, suspension/final graph, construction
evidence, parameter transfer site, capture/effect evidence,
ownership/consumption/close state, action evidence, and target adapters rather
than expose the compiler-session side table or private token. This realizes
`TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-001`,
`TOPAL-GENERATOR-FUNCTION-PARAMETER-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-GENERATOR-FOREACH-001` for compiler increment 5r.

## TOPAL-COMP-CUSTOM-GENERATOR-PARAMETER-CLOSE-001 — Closing a transferred custom parameter

The checked compiler shall extend the exact custom Generator parameter
specialization of `TOPAL-COMP-CUSTOM-GENERATOR-PARAMETER-001` to a
statement-free Unit function body that leaves the transferred parameter
unconsumed. Function exit shall consume and close that same suspended
continuation. The caller binding shall remain consumed, and neither caller nor
callee shall subsequently traverse or resume it.

The checked close node shall retain the callee parameter together with the
call-specialized custom construction provenance: declaration identity, exact
Character, directions, suspension/final graph, and ownership edge. Its close
domain shall be the lexical root namespace of the admitted function, separate
from the qualified generator declaration provenance. Distinct calls shall retain
distinct close provenance.

Because the admitted yield result is discarded and the exact continuation has
no handler, local state, cleanup, effect, or work after suspension, O0 lowering
may erase close delivery and final Unit after proving the explicit close node.
This erasure is semantic lowering, not an LLVM optimization. On Linux x86-64,
LLVM `fastcc` shall choose placement for the private `i32` Generator token
parameter; the backend shall hard-code no System V register placement. DWARF
and GDB shall preserve the Generator parameter and caller/callee frames through
the close edge.

Handled or bound-yield close, multiple yields, local state, cleanup/effects,
non-Unit results, explicit return, additional body statements, traversal before
exit, static/anonymous/recursive/nested/multiple-parameter calls,
direct-expression arguments, repeated caller use, and library paths shall remain
rejected before LLVM. The lowering shall introduce no Generator object or state
allocation, close dispatcher, callback, indirect call, unwind dependency, C/C++
runtime, other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or `topal-native/6`
revision. Future compiled-library metadata shall encode the canonical
declaration and directions, suspension/final graph, construction evidence,
parameter transfer and close sites, lexical close domain,
capture/effect/cleanup evidence, ownership/consumption/close state, and target
adapters rather than expose the compiler-session provenance or private token.
This realizes `TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-CLOSE-001`,
`TOPAL-GENERATOR-FUNCTION-PARAMETER-001`, and `TOPAL-GENERATOR-CLOSE-001` for
compiler increment 5s.

## TOPAL-COMP-CUSTOM-GENERATOR-CHARACTER-RESULT-PARAMETER-001 — Transferred final Character

The checked compiler shall extend the exact custom Generator parameter
specialization of `TOPAL-COMP-CUSTOM-GENERATOR-PARAMETER-001` to one named
`Generator Character Unit Character` parameter and Character function result.
The argument shall be a named root binding of the exact single-yield,
distinct-final-Character generator admitted by
`TOPAL-COMP-GENERATOR-FINAL-CHARACTER-001`. The function body shall be exactly
a result-valued Character foreach over that parameter with a
Character-to-Unit action. Argument evaluation shall consume the caller binding
and transfer sole ownership to the callee.

Successful traversal shall invoke the action once with the retained yielded
Character, resume with Unit, evaluate the separately retained final Character,
and return that final Character as the ordinary function result. It shall not
deliver close. The checked program shall preserve the exact Generator
classifier, declaration, yield and final values, suspension/resumption order,
action, and ownership transfer for each private call specialization. Distinct
calls shall not share yielded or final provenance, and the caller shall not
reuse the consumed Generator binding.

On Linux x86-64, LLVM `fastcc` shall choose placement for the private `i32`
Generator token parameter and ordinary Character descriptor result; the
backend shall hard-code no System V register or return placement. O0 lowering
shall expand the action and Unit resumption before materializing and returning
the final Character, independently of LLVM optimization. DWARF and GDB shall
expose the complete Generator classifier, callee-owned parameter, yielded
Character, final Character result, and caller/callee frames.

Static, anonymous, recursive, nested, multiple-parameter, additional-body,
direct-expression-argument, multiple-yield, generator-local, handled or
unconsumed, alternate-result, repeated-use, returned-continuation, and library
paths shall remain rejected before LLVM. The lowering shall introduce no
Generator object or state allocation, dispatcher, callback, indirect call,
unwind dependency, C/C++ runtime, other-language standard library, needed
library, dynamic relocation, public/library calling convention or Generator
ABI, or `topal-native/6` revision. Future compiled-library metadata shall
encode the canonical classifier and directions, declaration, distinct
yield/final graph, construction and parameter-transfer sites, action,
capture/effect evidence, ownership/consumption/close state, and target adapters
rather than expose the compiler-session specialization or private token. This
realizes
`TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-PARAMETER-001`,
`TOPAL-GENERATOR-FUNCTION-PARAMETER-001`,
`TOPAL-GENERATOR-FINAL-RETURN-001`, and `TOPAL-GENERATOR-FOREACH-001` for
compiler increment 5t.

## TOPAL-COMP-CUSTOM-GENERATOR-CHARACTER-RESULT-001 — Returning a continuation with final Character

The checked compiler shall extend the exact custom Generator result
specialization of `TOPAL-COMP-CUSTOM-GENERATOR-RESULT-001` to an ordinary
nonrecursive function whose result classifier is
`Generator Character Unit Character`. The function shall have exactly one
named Character parameter and a statement-free body that directly applies the
exact single-yield, distinct-final-Character generator admitted by
`TOPAL-COMP-GENERATOR-FINAL-CHARACTER-001` to that parameter. The top-level call
shall supply one exact Character and bind the returned continuation before
consuming it.

Function exit shall transfer the fresh continuation without close delivery.
The checked program shall associate the generator declaration, exact yielded
and final Characters, suspension/resumption graph, and ownership transfer with
the private call specialization. Caller traversal shall invoke the
Character-to-Unit action once, resume with Unit, then produce the separately
retained final Character. The caller binding shall be consumed exactly once,
and no result provenance shall be shared with another private specialization.

On Linux x86-64, LLVM `fastcc` shall choose placement for the ordinary
Character descriptor parameter and private `i32` Generator result; the backend
shall hard-code no System V register or return placement. The factory shall
return only the private ownership token. O0 caller lowering shall expand the
retained action and Unit resumption before materializing the final Character,
independently of LLVM optimization. DWARF and GDB shall expose the Character
factory parameter, complete Generator return classifier and value, yielded
Character, and caller/factory frames.

Static, anonymous, recursive, nested, multiple-parameter, statement-bearing,
non-parameter-derived, multiple-yield, generator-local, handled-close,
unbound-result, caller-close, parameter-transfer composition, repeated-use, and
library paths shall remain rejected before LLVM. The lowering shall introduce
no Generator object or state allocation, dispatcher, callback, indirect call,
unwind dependency, C/C++ runtime, other-language standard library, needed
library, dynamic relocation, public/library calling convention or Generator
ABI, or `topal-native/6` revision. Future compiled-library metadata shall encode
the canonical classifier and directions, declaration, distinct yield/final
graph, construction and function-result transfer sites, action, capture/effect
evidence, ownership/consumption/close state, and target adapters rather than
expose the compiler-session returned-provenance table or private token. This
realizes `TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-001`,
`TOPAL-GENERATOR-FUNCTION-RESULT-001`, `TOPAL-GENERATOR-FINAL-RETURN-001`, and
`TOPAL-GENERATOR-FOREACH-001` for compiler increment 5u.

## TOPAL-COMP-GENERATOR-STRING-INPUT-001 — Independent String initial input

The checked compiler shall admit a root custom generator whose one named
initial parameter is String while its directions are
`Generator Character Unit Unit`. Its body shall bind the result of
`empty? initial` to one named Boolean before one discarded yield of an exact
Character literal and a final Unit expression. One currently admitted String
expression shall start the generator, and the fresh result shall be bound and
consumed exactly once by a Character-to-Unit foreach action.

The checked program shall retain the String parameter independently of the
three Generator directions, the ordered pre-suspension block and Boolean
binding, the exact yielded Character, declaration and suspension spans, final
Unit, and the ownership edge. Generator application shall evaluate the String
operand once and execute its emptiness predicate before exposing the suspended
Generator binding. Traversal shall then invoke the action once, resume with
Unit, and complete with Unit. These steps shall hold with LLVM optimization
disabled and shall not depend on constant folding or dead-code elimination.

On Linux x86-64, the String descriptor and Boolean prefix value shall use the
existing `topal-native/6` representations, while the root-local Generator shall
remain a compiler-private `i32` ownership token. LLVM shall select target data
layout and instruction placement; the backend shall hard-code no AMD64
register convention. DWARF and GDB shall expose the String initial value while
the prefix executes, the Boolean prefix binding metadata, the complete
Generator value, the yielded Character, and the Topal entry frame.

Other initial classifiers, multiple parameters, alternate prefix operations,
unnamed or differently classified prefix bindings, computed or non-Character
yields, multiple yields, non-Unit resume or final directions, local state after
the yield, close handling, nested or ordinary-function construction, Generator
parameter/result transfer, repeated consumption, abandonment, libraries, and
external boundaries shall remain rejected before LLVM. The lowering shall
introduce no Generator object or semantic state allocation, dispatcher,
callback, indirect call, unwind dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library calling
convention or Generator ABI, or `topal-native/6` revision. Future
compiled-library metadata shall encode the initial classifier separately from
all three directions, the ordered pre-suspension operation and binding,
declaration, suspension/yield/final graph, construction/action sites,
capture/effect evidence, ownership/consumption/close state, and target adapters
rather than expose the private token or checked-program node layout. This
realizes `TOPAL-COMPILER-GENERATOR-STRING-INPUT-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`,
`TOPAL-GENERATOR-FOREACH-001`, and `TOPAL-STRING-EMPTY-PREDICATE-001` for
compiler increment 5v.

## TOPAL-COMP-GENERATOR-STRING-YIELD-001 — Independent String yield direction

The checked compiler shall admit a root custom generator with one named String
initial parameter and directions `Generator String Unit Unit`. Its body shall
contain one or more consecutive discarded yields, each yielding either that
initial parameter or an exact String literal, followed by a final Unit
expression. One currently admitted String expression shall start the generator,
and the fresh result shall be bound and consumed exactly once by a String-to-Unit
foreach action.

The checked program shall retain the initial classifier separately from the
three Generator directions and shall retain ordered per-yield value provenance,
declaration and suspension spans, final Unit, action, and ownership edge.
Application shall evaluate the initial String expression exactly once. Traversal
shall deliver that retained descriptor for an initial-parameter yield,
materialize each exact literal at its suspension point, invoke the action and
resume with Unit after every yield, and complete with Unit. This source order
shall hold with LLVM optimization disabled and shall not depend on constant
folding, inlining, or dead-code elimination.

On Linux x86-64, yielded values shall use the existing `topal-native/6` String
descriptor while the root-local Generator remains a compiler-private `i32`
ownership token. LLVM shall select target data layout and instruction placement;
the backend shall hard-code no AMD64 register convention. DWARF and GDB shall
expose the complete Generator and each yielded String while its action executes,
plus the Topal entry frame.

Other input, yield, or resume classifiers; final classifiers beyond the exact
String result admitted by `TOPAL-COMP-GENERATOR-FINAL-STRING-001`; multiple
parameters; computed yields; intervening or post-suspension body state beyond
the exact one-discard continuation admitted by
`TOPAL-COMP-GENERATOR-RESUME-DISCARD-001`; close handling; nested or
ordinary-function construction; Generator parameter/result transfer; repeated
consumption; abandonment; libraries; and external boundaries shall remain
rejected before LLVM. The lowering shall introduce no Generator object or
semantic state allocation, dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator ABI, or
`topal-native/6` revision. Future compiled-library metadata shall encode the
initial and direction classifiers independently, ordered per-yield values and
provenance, declaration, suspension/final graph, construction/action sites,
capture/effect evidence, ownership/consumption/close state, and target adapters
rather than expose the private token or checked-program node layout. This
realizes `TOPAL-COMPILER-GENERATOR-STRING-YIELD-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`,
`TOPAL-GENERATOR-FOREACH-001`, and `TOPAL-STRING-EMPTY-PREDICATE-001` for
compiler increment 5w.

## TOPAL-COMP-GENERATOR-FINAL-STRING-001 — Distinct final String

The checked compiler shall admit a root custom generator with one named String
initial parameter and directions `Generator String Unit String`. Its body shall
contain one or more consecutive discarded String yields in the forms admitted
by `TOPAL-COMP-GENERATOR-STRING-YIELD-001`, followed by one exact String literal
as its distinct final expression. One currently admitted String expression
shall start the generator, and the fresh result shall be bound and consumed
exactly once by a String-to-Unit foreach action whose expression result is the
final String.

The checked program shall retain the final String expression and classifier
separately from the initial value, ordered yield provenance, Unit resume
direction, declaration and suspension spans, action, and ownership edge.
Application shall evaluate the initial expression exactly once. Traversal shall
deliver every yield, invoke its action, resume with Unit, and only then
materialize and return the final String. This order shall hold with LLVM
optimization disabled and shall not depend on folding, inlining, or dead-code
elimination.

On Linux x86-64, the input, yielded, and final values shall use the existing
`topal-native/6` String descriptor while the root-local Generator remains a
compiler-private `i32` ownership token. LLVM shall select target data layout and
instruction placement; the backend shall hard-code no AMD64 register
convention. DWARF and GDB shall expose the complete
`Generator String Unit String` classifier and value, the yielded String during
its action, the final expression source location, and the Topal entry frame.

Nonliteral or initial-derived final values; other input, yield, resume, or final
classifiers; multiple parameters; computed yields; intervening body state;
close handling; nested or ordinary-function construction; Generator
parameter/result transfer; repeated consumption; abandonment; libraries; and
external boundaries shall remain rejected before LLVM. The lowering shall
introduce no Generator object or semantic state allocation, dispatcher,
callback, indirect call, unwind dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library calling
convention or Generator ABI, or `topal-native/6` revision. Future
compiled-library metadata shall encode the initial and direction classifiers
independently, ordered yield provenance, the distinct final-value expression
and provenance, declaration, suspension/final graph, construction/action sites,
capture/effect evidence, ownership/consumption/close state, and target adapters
rather than expose the private token or checked-program node layout. This
realizes `TOPAL-COMPILER-GENERATOR-FINAL-STRING-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`,
`TOPAL-GENERATOR-FINAL-RETURN-001`, and `TOPAL-GENERATOR-FOREACH-001` for
compiler increment 5x.

## TOPAL-COMP-GENERATOR-RESUME-DISCARD-001 — Post-resume discarded computation

The checked compiler shall admit a root custom generator with one named String
initial parameter and directions `Generator String Unit Unit` whose body
extends the yield forms of `TOPAL-COMP-GENERATOR-STRING-YIELD-001` with exactly
one discarded `empty? initial` computation. At least one yield shall precede
that computation and at least one yield shall follow it. The body shall end in
Unit. One currently admitted String expression shall start the generator, and
the fresh result shall be bound and consumed exactly once by a String-to-Unit
foreach action.

The checked program shall retain the computation as a typed continuation block
with its source span and exact successful-resumption ordinal, separately from
the initial value, ordered yield provenance, action, final Unit, and ownership
edge. Application shall evaluate the initial expression exactly once. Traversal
shall deliver and act on each preceding yield, resume it with Unit, execute the
discarded emptiness computation against the captured initial String, and only
then reach the following suspension. This order shall hold with LLVM
optimization disabled and shall not depend on folding, inlining, or dead-code
elimination.

On Linux x86-64, the initial and yielded values shall use the existing
`topal-native/6` String descriptor while the root-local Generator remains a
compiler-private `i32` ownership token. LLVM shall select target data layout and
instruction placement; the backend shall hard-code no AMD64 register
convention. DWARF and GDB shall expose the complete
`Generator String Unit Unit` classifier and value, each yielded String during
its action, the captured initial String during the post-resume computation, its
source location, and the Topal entry frame.

Bindings, more than one continuation computation, a computation before the
first or after the final yield, operands other than the initial parameter,
computed continuation expressions, non-Unit final values, other input, yield,
or resume classifiers, close handling, nested or ordinary-function
construction, Generator parameter/result transfer, repeated consumption,
abandonment, libraries, and external boundaries shall remain rejected before
LLVM. The lowering shall introduce no Generator object or semantic state
allocation, dispatcher, callback, indirect call, unwind dependency, C/C++
runtime, other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or `topal-native/6`
revision. Future compiled-library metadata shall encode the initial and
direction classifiers independently, ordered yields, the continuation block
and successful-resumption ordinal, expression provenance and source sites,
declaration/suspension/final graph, action, capture/effect evidence,
ownership/consumption/close state, and target adapters rather than expose the
private token or checked-program node layout. This realizes
`TOPAL-COMPILER-GENERATOR-RESUME-DISCARD-001`,
`TOPAL-GENERATOR-BODY-STATEMENT-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-GENERATOR-FOREACH-001` for compiler increment 5y.

## TOPAL-COMP-GENERATOR-EXPLICIT-RETURN-001 — Explicit return before suspension

The checked compiler shall admit a root custom generator with one named String
initial parameter and directions `Generator String Unit String` whose body
consists only of `return` applied to one exact String literal. One currently
admitted String expression shall start the generator, and the fresh result
shall be bound and consumed exactly once by a String-to-Unit foreach action
whose expression result is the explicitly returned String.

The checked program shall distinguish the explicit-return keyword and source
site from the typed String result expression, implicit final expressions,
empty ordered yield/continuation lists, action, and ownership edge. Application
shall evaluate the initial expression exactly once even when the return does
not reference it. Traversal shall recognize completion before the first
suspension, invoke the foreach action zero times, and directly materialize and
return the exact String. This order shall hold with LLVM optimization disabled
and shall not depend on folding, inlining, or dead-code elimination.

On Linux x86-64, the initial and returned values shall use the existing
`topal-native/6` String descriptor while the root-local Generator remains a
compiler-private `i32` ownership token. A debug-only pointer shadow may keep
the in-scope initial parameter inspectable without becoming semantic Generator
state. LLVM shall select target data layout and instruction placement; the
backend shall hard-code no AMD64 register convention. DWARF and GDB shall
expose the complete `Generator String Unit String` classifier and value, the
initial String at the explicit-return source location, and the Topal entry
frame. They shall not fabricate a yielded action value when no suspension is
reached.

Implicit zero-yield final expressions, nonliteral or initial-derived returns,
explicit return after a suspension beyond the exact case admitted by
`TOPAL-COMP-GENERATOR-RETURN-AFTER-YIELD-001`, Unit or other return classifiers,
additional body statements, multiple parameters, other input, yield, or resume
classifiers, close handling, nested or ordinary-function construction,
Generator parameter/result transfer, repeated consumption, abandonment,
libraries, and external boundaries shall remain rejected before LLVM. The
lowering shall introduce no Generator object or semantic state allocation,
dispatcher, callback, indirect call, unwind dependency, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or `topal-native/6`
revision. Future compiled-library metadata shall encode explicit versus
implicit completion, return expression and keyword provenance, reachability
and empty-yield evidence, independent initial and direction classifiers,
declaration/construction/action sites, captures/effects,
ownership/consumption/close state, and target adapters rather than expose the
private token, debug shadow, or checked-program node layout. This realizes
`TOPAL-COMPILER-GENERATOR-EXPLICIT-RETURN-001`,
`TOPAL-GENERATOR-EXPLICIT-RETURN-001`, `TOPAL-GENERATOR-FINAL-RETURN-001`, and
`TOPAL-GENERATOR-FOREACH-001` for compiler increment 5z.

## TOPAL-COMP-GENERATOR-RETURN-AFTER-YIELD-001 — Explicit return after resumption

The checked compiler shall admit a root custom generator with one named String
initial parameter and directions `Generator String Unit String` whose body
consists only of one discarded `yield initial` followed by `return` applied to
one exact String literal. One currently admitted String expression shall start
the generator, and the fresh result shall be bound and consumed exactly once by
a String-to-Unit foreach action whose expression result is the explicitly
returned String.

The checked program shall retain the initial-parameter yield and suspension,
the explicit-return keyword and source site, and the typed final String as
separate ordered provenance. Application shall evaluate the initial expression
exactly once. Traversal shall invoke the action exactly once with that retained
String, resume the generator with Unit, and only then materialize and return the
exact String literal. This order shall hold with LLVM optimization disabled and
shall not depend on folding, inlining, or dead-code elimination.

On Linux x86-64, the initial, yielded, and returned values shall use the
existing `topal-native/6` String descriptor while the root-local Generator
remains a compiler-private `i32` ownership token. A debug-only pointer shadow
may keep the initial parameter inspectable at the return site without becoming
semantic Generator state. LLVM shall select target data layout and instruction
placement; the backend shall hard-code no AMD64 register convention. DWARF and
GDB shall expose the complete `Generator String Unit String` classifier and
value, the yielded String during its action, the initial String at the explicit
return, and the Topal entry frame.

Literal or computed yields other than the initial parameter, multiple yields,
nonliteral or initial-derived returns, intervening statements, Unit or other
direction classifiers, multiple parameters, close handling, nested or
ordinary-function construction, Generator parameter/result transfer, repeated
consumption, abandonment, libraries, and external boundaries shall remain
rejected before LLVM. The lowering shall introduce no Generator object or
semantic state allocation, dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator ABI, or
`topal-native/6` revision. Future compiled-library metadata shall encode
explicit completion, ordered yield/resume/return provenance and reachability,
independent initial and direction classifiers, declaration/construction/action
sites, captures/effects, ownership/consumption/close state, and target adapters
rather than expose the private token, debug shadow, or checked-program node
layout. This realizes `TOPAL-COMPILER-GENERATOR-RETURN-AFTER-YIELD-001`,
`TOPAL-GENERATOR-EXPLICIT-RETURN-001`, `TOPAL-GENERATOR-RESUMPTION-001`, and
`TOPAL-GENERATOR-FOREACH-001` for compiler increment 5aa.

## TOPAL-COMP-GENERATOR-BOOLEAN-001 — Boolean generator directions

The checked compiler shall admit a root custom generator with one named
Boolean initial parameter and directions `Generator Boolean Unit Boolean` whose
body consists only of one discarded `yield initial` followed by `not initial`
as its distinct final expression. One currently admitted Boolean expression
shall start the generator, and the fresh result shall be bound and consumed
exactly once by a Boolean-to-Unit foreach action whose expression result is the
final Boolean.

The checked program shall retain the Boolean initial classifier separately
from all three Generator directions, the initial-parameter yield and
suspension, the typed final negation, action, and ownership edge. Application
shall evaluate the initial expression exactly once. Traversal shall invoke the
action exactly once with that retained Boolean, resume the generator with Unit,
and only then evaluate the final negation against the captured initial value.
This order shall hold with LLVM optimization disabled and shall not depend on
folding, inlining, or dead-code elimination.

Unoptimized LLVM IR shall retain the action negation before the final negation.
Mandatory target instruction selection may omit the unused action result only
because Boolean negation is total and effect-free; this shall not generalize to
an action with an observable effect. The independently emitted debug lifetime
anchor shall keep the yielded value inspectable even when no machine
instruction is selected for that pure result.

On Linux x86-64, each Boolean shall use LLVM `i1` in the existing private
`topal-native/6` representation while the root-local Generator remains a
compiler-private `i32` ownership token. A debug-only aligned `i1` stack shadow
may anchor the yielded value's source lifetime without becoming semantic
Generator state. LLVM shall select physical register or stack placement and
instruction selection; the backend shall hard-code no AMD64 register
convention. DWARF and GDB shall expose the complete
`Generator Boolean Unit Boolean` classifier and value, the yielded Boolean in
the foreach action scope, the captured initial Boolean at the final expression,
and the Topal entry frame.

Literal, computed, or multiple yields; a final expression other than
`not initial`; additional body statements; multiple parameters; other input,
yield, resume, or final classifiers; close handling; nested or
ordinary-function construction; Generator parameter/result transfer; repeated
consumption; abandonment; libraries; and external boundaries shall remain
rejected before LLVM. The lowering shall introduce no Generator object or
semantic state allocation, Boolean helper, dispatcher, callback, indirect
call, unwind dependency, C/C++ runtime, other-language standard library,
needed library, dynamic relocation, public/library calling convention or
Generator ABI, or `topal-native/6` revision. Future compiled-library metadata
shall encode independent initial and direction classifiers, ordered Boolean
expression and yield/resume/final provenance, declaration/construction/action
sites, captures/effects, ownership/consumption/close state, and target adapters
rather than expose the private token, debug shadow, or checked-program node
layout. This realizes `TOPAL-COMPILER-GENERATOR-BOOLEAN-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-FOREACH-001`, and
`TOPAL-GENERATOR-FINAL-RETURN-001` for compiler increment 5ab.

## TOPAL-COMP-GENERATOR-INT-001 — Arbitrary-precision Int generator directions

The checked compiler shall admit a root custom generator with one named Int
initial parameter and directions `Generator Int Unit Int` whose body consists
only of one discarded `yield initial` followed by `initial + 1` as its distinct
final expression. One currently admitted Int expression shall start the
generator, and the fresh result shall be bound and consumed exactly once by an
Int-to-Unit foreach action consisting only of a discarded `value + 1`.

The checked program shall retain the Int initial classifier separately from all
three Generator directions, the initial-parameter yield and suspension, both
typed additions and their exact-one operands, action, and ownership edge.
Application shall evaluate the initial expression exactly once. Traversal shall
invoke the action addition exactly once with that retained Int, resume the
generator with Unit, and only then evaluate the final addition against the
captured initial value. Both additions shall use the canonical arbitrary-
precision Int runtime and preserve all values exactly. This order and explicit
allocation-failure behavior shall hold with LLVM optimization disabled and
shall not depend on folding, inlining, or dead-code elimination.

Unoptimized LLVM IR shall retain two direct Int-addition calls in source order.
Unlike the total register-only Boolean action in
`TOPAL-COMP-GENERATOR-BOOLEAN-001`, LLVM shall not omit the discarded Int
action: arbitrary-precision addition can allocate and therefore can reach the
required platform-failure path. LLVM may optimize inside or around the calls
only when exact values, evaluation order, and allocation failure remain
observationally equivalent.

On Linux x86-64, every Int shall retain the existing private immutable
`topal-native/6` canonical sign-and-magnitude pointer representation while the
root-local Generator remains a compiler-private `i32` ownership token. LLVM
shall select pointer placement and call lowering from the target triple and
data layout; the backend shall hard-code no AMD64 register convention. Two
debug-only aligned pointer slots may anchor the yielded and captured-initial
source lifetimes without becoming semantic Generator state. DWARF, the Topal
GDB printer, and GDB shall expose the complete `Generator Int Unit Int`
classifier and value, the exact yielded Int in the foreach action scope, the
exact captured initial Int at the final expression, and the Topal entry frame.

Literal, computed, or multiple yields; a final expression other than
`initial + 1`; additional body statements; multiple parameters; other input,
yield, resume, or final classifiers; close handling; nested or
ordinary-function construction; Generator parameter/result transfer; repeated
consumption; abandonment; libraries; and external boundaries shall remain
rejected before LLVM. The lowering shall introduce no semantic Generator
object or state allocation, dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator ABI, or
`topal-native/6` revision. Existing Int values and arithmetic allocations shall
remain wholly owned by the Topal runtime and Linux syscall layer. Future
compiled-library metadata shall encode independent initial and direction
classifiers, ordered Int expression and yield/resume/final provenance, exact
numeric requirements, declaration/construction/action sites, captures/effects
including allocation failure, ownership/consumption/close state, native-
representation identity, and target adapters rather than expose the private
token, debug slots, Int object layout, or checked-program node layout. This
realizes `TOPAL-COMPILER-GENERATOR-INT-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-GENERATOR-FINAL-RETURN-001` for compiler increment 5ac.

## TOPAL-COMP-GENERATOR-RATIONAL-001 — Exact Rational generator directions

The checked compiler shall admit a root custom generator with one named
Rational initial parameter and directions `Generator Rational Unit Rational`
whose body consists only of one discarded `yield initial` followed by
`initial + (Rational (1, 3))` as its distinct final expression. One currently
admitted Rational expression shall start the generator, and the fresh result
shall be bound and consumed exactly once by a Rational-to-Unit foreach action
consisting only of a discarded `value + (Rational (1, 3))`.

The checked program shall retain the Rational initial classifier separately
from all three Generator directions, the initial-parameter yield and
suspension, both typed additions, their exact one-third constructor provenance,
action, and ownership edge. Application shall evaluate and canonically
construct the initial expression exactly once. Traversal shall construct the
action addend and invoke the action addition exactly once with that retained
Rational, resume the generator with Unit, and only then construct the final
addend and evaluate the final addition against the captured initial value. All
constructors and additions shall use the canonical exact Rational and
arbitrary-precision Int runtime. This order and explicit allocation-failure
behavior shall hold with LLVM optimization disabled and shall not depend on
folding, inlining, or dead-code elimination.

Unoptimized LLVM IR shall retain three direct canonical Rational-construction
calls and two direct Rational-addition calls in source order. LLVM shall not
omit the discarded action construction or addition: both may allocate and
therefore can reach the required platform-failure path. LLVM may optimize
inside or around these calls only when exact canonical values, evaluation
order, and allocation failure remain observationally equivalent.

On Linux x86-64, every Rational shall retain the existing private immutable
`topal-native/6` pointer representation containing canonical arbitrary-
precision Int numerator and positive denominator pointers, while the root-local
Generator remains a compiler-private `i32` ownership token. LLVM shall select
pointer placement and call lowering from the target triple and data layout; the
backend shall hard-code no AMD64 register convention. Two debug-only aligned
pointer slots may anchor the yielded and captured-initial source lifetimes
without becoming semantic Generator state. DWARF, the Topal GDB printer, and
GDB shall expose the complete `Generator Rational Unit Rational` classifier and
value, the exact yielded Rational in the foreach action scope, the exact
captured initial Rational at the final expression, and the Topal entry frame.

Literal, converted, computed, or multiple yields; different Rational addends;
a final expression other than the exact declared addition; additional body
statements; multiple parameters; other input, yield, resume, or final
classifiers; close handling; nested or ordinary-function construction;
Generator parameter/result transfer; repeated consumption; abandonment;
libraries; and external boundaries shall remain rejected before LLVM. The
lowering shall introduce no semantic Generator object or state allocation,
dispatcher, callback, indirect call, unwind dependency, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or `topal-native/6`
revision. Existing Rational/Int values and arithmetic allocations shall remain
wholly owned by the Topal runtime and Linux syscall layer. Future compiled-
library metadata shall encode independent initial and direction classifiers,
ordered Rational expression and yield/resume/final provenance, exact canonical
numerator/denominator and constructor requirements,
declaration/construction/action sites, captures/effects including allocation
failure, ownership/consumption/close state, native-representation identity, and
target adapters rather than expose the private token, debug slots,
Rational/Int object layouts, or checked-program node layout. This realizes
`TOPAL-COMPILER-GENERATOR-RATIONAL-001`, `TOPAL-COMPILER-EXACT-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-GENERATOR-FINAL-RETURN-001` for compiler increment 5ad.

## TOPAL-COMP-GENERATOR-UNIT-001 — Payload-free Unit generator directions

The checked compiler shall admit a root custom generator with one named Unit
initial parameter and directions `Generator Unit Unit Unit` whose body consists
only of one discarded `yield initial` followed by `()` as its final expression.
One currently admitted Unit expression shall start the generator, and the fresh
result shall be bound and consumed exactly once by a Unit-to-Unit foreach action
consisting only of its named yielded parameter as an identity expression.

The checked program shall retain the Unit initial classifier separately from
all three Generator directions, the initial-parameter yield and suspension, the
named identity action, Unit resumption, distinct final Unit, and ownership edge.
Application shall evaluate the initial expression exactly once. Traversal shall
evaluate the action exactly once with the yielded Unit, resume the generator
with Unit, and only then evaluate the final Unit. This order shall hold with
LLVM optimization disabled and shall not depend on folding, inlining, or dead-
code elimination.

Because Unit has exactly one value and no runtime payload, unoptimized semantic
LLVM IR shall introduce no action call, payload operation, allocation, or
Generator runtime operation. Erasing those payload operations is a compiler
representation decision, not an LLVM optimization, and shall preserve the
checked action occurrence and source order. Debug LLVM IR shall contain two
aligned private `i8` lifetime slots: one anchors the yield and action sites, and
one initialized slot anchors the captured initial and final-expression site.
These slots and their stores shall be debug-only and shall not become semantic
Generator or Unit state. LLVM may eliminate or transform the debug-only
artifacts when permitted by the selected debug/optimization policy.

On Linux x86-64, Unit shall have no semantic native payload while the root-
local Generator remains a compiler-private `i32` ownership token. LLVM shall
select placement and call lowering from the target triple and data layout; the
backend shall hard-code no AMD64 register convention. DWARF and GDB shall
expose the complete `Generator Unit Unit Unit` classifier and value, the Unit
yielded into the foreach action, the captured initial Unit at the final
expression, ordered yield/action/resumption/final source locations, and the
Topal entry frame.

Literal or multiple yields; a final expression other than exact `()`;
additional body statements; an action other than the named identity expression;
multiple parameters; other input, yield, resume, or final classifiers; close
handling; nested or ordinary-function construction; Generator parameter/result
transfer; repeated consumption; abandonment; libraries; and external
boundaries shall remain rejected before LLVM. The lowering shall introduce no
semantic Generator object or state allocation, dispatcher, callback, indirect
call, unwind dependency, C/C++ runtime, other-language standard library, needed
library, dynamic relocation, public/library calling convention or Generator
ABI, or `topal-native/6` revision. Output shall remain wholly owned by the Topal
runtime and Linux syscall layer. Future compiled-library metadata shall encode
independent initial and direction classifiers, the zero-payload Unit identity,
ordered yield/action/resume/final provenance, representation erasure guarantees,
declaration/construction/action sites, captures/effects, ownership/consumption/
close state, native-representation identity, and target adapters rather than
expose the private token, debug slots, or checked-program node layout. This
realizes `TOPAL-COMPILER-GENERATOR-UNIT-001`,
`TOPAL-GENERATOR-DECLARATION-001`, and `TOPAL-GENERATOR-FOREACH-001` for
compiler increment 5ae.

## TOPAL-COMP-GENERATOR-OPTIONAL-001 — Nominal Optional generator directions

The checked compiler shall admit a root custom generator with one named
`Optional Int` initial parameter and directions
`Generator Optional Int Unit Optional Int` whose body consists only of one
discarded `yield initial` followed by `None Int` as its final expression. One
currently admitted `Optional Int` expression shall start the generator; the
shared regression uses `(Some 7)`. The fresh result shall be bound and consumed
exactly once by a foreach action consisting only of the discarded equality
`candidate = (Some 7)` for its named yielded parameter.

The checked program shall retain the nominal Optional classifier and its Int
payload classifier separately from all three Generator directions, the
initial-parameter yield and suspension, the exact Some construction and
equality action, Unit resumption, distinct final None construction, and
ownership edge. Application shall construct the initial Optional exactly once.
Traversal shall pass that same immutable Optional value to the action, evaluate
the exact right operand and equality once, resume the generator with Unit, and
only then construct the final None value. This order shall hold with LLVM
optimization disabled and shall not depend on folding, inlining, or dead-code
elimination.

On Linux x86-64, the existing compiler-private Optional representation shall
remain an aligned pointer to Topal-owned tagged storage with an Int payload
pointer for Some and a null payload for None. The root-local Generator shall
remain a compiler-private `i32` ownership token. LLVM shall select placement
and call lowering from the target triple and data layout; the backend shall
hard-code no AMD64 register convention. Two aligned debug-only pointer shadows
shall preserve the yielded action value and captured initial lifetime. DWARF
and GDB shall expose the complete `Generator Optional Int Unit Optional Int`
classifier and value, the yielded `Some 7`, the captured initial `Some 7`,
ordered yield/action/resumption/final source locations, and the Topal entry
frame.

Literal or multiple yields; a final expression other than exact `None Int`;
additional body statements; another Optional payload, action, input, yield,
resume, or final classifier; close handling; nested or ordinary-function
construction; Generator parameter/result transfer; repeated consumption;
abandonment; libraries; and external boundaries shall remain
rejected before LLVM. The lowering shall introduce no semantic Generator
object or state allocation, dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator ABI, or
`topal-native/6` revision. Optional storage, allocation, equality, display, and
Linux syscalls shall remain wholly owned by Topal. Future compiled-library
metadata shall encode independent initial and direction classifiers, the
nominal Optional and payload identities, exact alternatives and action
operands, ordered yield/action/resume/final provenance, allocation-failure
effects, declaration and construction sites, captures/effects, ownership/
consumption/close state, native-representation identity, and target adapters
rather than expose the private token, debug slots, Optional/Int object layouts,
or checked-program node layout. This realizes
`TOPAL-COMPILER-GENERATOR-OPTIONAL-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-TYPE-OPTIONAL-BOUNDARY-001` for compiler increment 5af.

## TOPAL-COMP-GENERATOR-RESULT-001 — Exact Result generator directions

The checked compiler shall admit a root custom generator with one named
`Result (Rational, lang arithmetic ArithmeticErrorCode)` initial parameter and
matching yield and final directions, with Unit resumption. Its body shall
consist only of one discarded `yield initial` followed by
`initial / (Rational 0)`. The shared regression shall start the generator with
the exact proven-success expression `Rational 1`; the fresh result shall be
bound and consumed exactly once by a foreach action consisting only of the
discarded self-equality `candidate = candidate` for its named yielded
parameter.

The checked program shall retain the nominal Result and Rational success
identities separately in all three Generator directions, the proven-success
promotion, initial-parameter yield and suspension, reflexive action, Unit
resumption, structured fallible final division, source-located error
provenance, declaration provenance, and ownership edge. Application shall
evaluate and promote Rational 1 exactly once. Traversal shall pass that same
immutable successful Result to the action, prove its self-equality without
inspecting or duplicating the payload, resume with Unit, and only then perform
the exact division by zero and produce the structured failure Result. This
order shall hold with LLVM optimization disabled and shall not depend on
folding, inlining, or dead-code elimination.

On Linux x86-64, Result and Rational shall retain their existing Topal-owned
compiler-private pointer representations, and the root-local Generator shall
remain a compiler-private `i32` ownership token. LLVM shall derive placement,
alignment, and call lowering from the target triple and data layout; the
backend shall hard-code no AMD64 register convention. Two aligned debug-only
pointer shadows and four lifetime/source anchor stores shall keep the yielded
action value and captured initial inspectable. DWARF and GDB shall expose the
complete Result-bearing Generator classifier and value, the yielded successful
Rational 1 Result, captured initial Result, ordered yield/action/resumption/
final source locations, and the Topal entry frame.

Another Result success/error classifier, input expression or value, direction,
yield, final operation, divisor, or action; multiple yields; additional body
statements; general inlined Result projection/propagation; close handling;
nested or ordinary-function construction; Generator parameter/result transfer;
repeated consumption; abandonment; libraries; and external boundaries shall
remain rejected before LLVM. The lowering shall introduce no semantic
Generator object or state allocation, dispatcher, callback, indirect call,
unwind dependency, C/C++ runtime, other-language standard library, needed
library, dynamic relocation, public/library calling convention or Generator/
Result ABI, or `topal-native/6` revision. Result/Rational storage, allocation,
failure construction, display, and Linux syscalls shall remain wholly owned by
Topal. Future compiled-library metadata shall encode independent initial and
direction classifiers, nominal Result/success/error identities, success
evidence, fallible-operation and source-error provenance, ordered yield/action/
resume/final provenance, allocation-failure effects, declaration/construction
sites, captures/effects, ownership/consumption/close state, native-
representation identity, and target adapters rather than expose the private
token, debug slots, pointer/object layouts, or checked-program node layout.
This realizes `TOPAL-COMPILER-GENERATOR-RESULT-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-TYPE-RESULT-001` for compiler increment 5ak.

## TOPAL-COMP-GENERATOR-COMPARISON-001 — Exact Comparison generator directions

The checked compiler shall admit a root custom generator with one named
`Comparison` initial parameter and directions
`Generator Comparison Unit Comparison`. Its body shall consist only of one
discarded `yield initial` followed by `3 <=> 2` as its final expression. The
shared regression shall start the generator with the exact expression
`1 <=> 2`; the fresh result shall be bound and consumed exactly once by a
foreach action consisting only of the discarded
`comparison = (1 <=> 2)` for its named yielded parameter.

The checked program shall retain the language-defined nominal `Comparison`
identity separately in all three Generator directions, both ordered Int
operands and three-way operations, the initial-parameter yield and suspension,
the exact equality action, Unit resumption, distinct final comparison,
declaration provenance, and ownership edge. Application shall evaluate
`1 <=> 2` exactly once. Traversal shall pass that same immutable Less value to
the action, independently evaluate the action's `1 <=> 2` once and compare the
two nominal values, resume the generator with Unit, and only then evaluate
`3 <=> 2` to produce Greater. This order shall hold with LLVM optimization
disabled and shall not depend on folding, inlining, or dead-code elimination.

On Linux x86-64, `Comparison` shall retain its existing compiler-private signed
`i32` values for Less, Equal, and Greater, and the root-local Generator shall
remain a compiler-private `i32` ownership token. LLVM shall derive placement,
alignment, and call lowering from the target triple and data layout; the
backend shall hard-code no AMD64 register convention. Two aligned debug-only
`i32` shadows and four lifetime/source anchor stores shall keep the yielded
action value and captured initial inspectable. DWARF and GDB shall expose the
complete `Generator Comparison Unit Comparison` classifier and value, the
yielded Less value, captured initial Less value, ordered yield/action/
resumption/final source locations, and the Topal entry frame.

Another input, action, final comparison, operand, direction, yield, or value;
multiple yields; additional body statements; close handling; nested or
ordinary-function construction; Generator parameter/result transfer; repeated
consumption; abandonment; libraries; and external boundaries shall remain
rejected before LLVM. The lowering shall introduce no semantic Generator
object or state allocation, dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator/Comparison
ABI, or `topal-native/6` revision. Int/Comparison allocation, comparison,
equality, display, and Linux syscalls shall remain wholly owned by Topal.
Future compiled-library metadata shall encode independent initial and direction
classifiers, nominal Comparison identity and ordered alternatives, operand and
operation provenance, ordered yield/action/resume/final provenance,
allocation-failure effects, declaration/construction sites, captures/effects,
ownership/consumption/close state, native-representation identity, and target
adapters rather than expose the private token, debug slots, scalar tags, or
checked-program node layout. This realizes
`TOPAL-COMPILER-GENERATOR-COMPARISON-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-DECISION-COMPARISON-001` for compiler increment 5al.

## TOPAL-COMP-GENERATOR-NESTED-OPTIONAL-001 — Recursive Optional product directions

The checked compiler shall admit a root custom generator with one named
`Optional (Int, String)` initial parameter and directions
`Generator Optional (Int, String) Unit Optional (Int, String)`. Its body shall
consist only of one discarded `yield initial` followed by
`Some (8, "done")`. The shared regression shall start the generator with exact
`Some (7, "item")`; the fresh result shall be bound and consumed exactly once
by a foreach action consisting only of the discarded
`candidate = (Some (7, "item"))` for its named yielded parameter.

The checked program shall retain the nominal Optional identity, Some
alternative, positional-product arity and order, and Int and String field
identities separately in all three Generator directions. It shall also retain
the initial-parameter yield and suspension, field-wise Optional equality
action, Unit resumption, distinct final product, declaration provenance, and
ownership edge. Application shall evaluate both initial payload fields once in
source order and construct one Optional. Traversal shall pass that same
immutable value to the action, construct the action operand once, compare tags
before observing Some payloads, compare payload fields in source order, resume
with Unit, and only then construct `Some (8, "done")`. This order shall hold at
LLVM O0 without relying on folding, inlining, or dead-code elimination.

On Linux x86-64, Optional shall retain its existing Topal-owned tagged pointer
header. Each admitted Some product payload shall use a Topal-owned aligned
16-byte allocation containing the existing Int and String pointers in source
order. The root-local Generator shall remain a compiler-private `i32` ownership
token. LLVM shall derive placement, alignment, and call lowering from the
target triple and data layout; the backend shall hard-code no AMD64 register
convention. Two aligned debug-only Optional pointer shadows shall preserve the
yielded action value and captured initial lifetime. DWARF and GDB shall expose
the complete recursive Generator classifier and value, the Optional product
classifier, ordered payload fields, both yielded/captured `Some (7, "item")`
values, ordered yield/action/resumption/final source locations, and the Topal
entry frame.

Another Optional payload, product arity, field order, input, action, final,
direction, yield, or value; None; multiple yields; additional body statements;
close handling; nested construction; ordinary-function construction or
Generator parameter/result transfer outside
`TOPAL-COMP-GENERATOR-NESTED-FUNCTION-BOUNDARY-001`; repeated consumption;
abandonment; libraries; and external
boundaries shall remain rejected before LLVM. The lowering shall introduce no
semantic Generator object or state allocation, generic Optional/product
dispatcher, callback, indirect call, unwind dependency, C/C++ runtime, other-
language standard library, needed library, dynamic relocation, public/library
calling convention or Generator/Optional/product ABI, or `topal-native/6`
revision. Optional, product, Int, and String allocation, equality, display, and
Linux syscalls shall remain wholly owned by Topal. Future compiled-library
metadata shall encode recursive classifiers, independent directions, nominal
alternatives, product arity/order, field identities, equality and ordered
yield/action/resume/final provenance, allocation-failure effects, declaration/
construction sites, captures/effects, ownership/consumption/close state,
native-representation identity, and target adapters rather than expose private
tokens, debug slots, object layouts, or checked-program nodes. This realizes
`TOPAL-COMPILER-GENERATOR-NESTED-OPTIONAL-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`,
`TOPAL-TYPE-OPTIONAL-CONSTRUCT-001`, and `TOPAL-TYPE-PRODUCT-001` for compiler
increment 5am.

## TOPAL-COMP-GENERATOR-NESTED-RESULT-001 — Recursive Result product directions

The checked compiler shall admit a root custom generator with one named
`Result ((Int, String), lang arithmetic ArithmeticErrorCode)` initial parameter
and the same yield and result directions with Unit resumption. Its body shall
consist only of one discarded `yield initial` followed by `(8, "done")`. The
shared regression shall start the generator with exact `(7, "item")`; the fresh
result shall be bound and consumed exactly once by a foreach action consisting
only of the discarded `candidate = (7, "item")` for its named yielded
parameter. Each product shall satisfy its explicit Result contract as an
implicit success value.

The checked program shall retain the nominal Result and arithmetic-error
vocabulary identities, success alternative, positional-product arity and
order, and Int and String field identities separately in all three Generator
directions. It shall also retain the initial-parameter yield and suspension,
success-gated field equality action, Unit resumption, distinct final product,
declaration provenance, and ownership edge. Application shall evaluate both
initial payload fields once in source order and construct one successful
Result. Traversal shall pass that same immutable value to the action, construct
the action success operand once, inspect both success/error tags before
observing success payloads, compare payload fields in source order, resume with
Unit, and only then construct the successful `(8, "done")`. This order shall
hold at LLVM O0 without relying on folding, inlining, or dead-code elimination.

On Linux x86-64, Result shall retain its existing Topal-owned tagged pointer
header. Each admitted successful product payload shall use a Topal-owned
aligned 16-byte allocation containing the existing Int and String pointers in
source order. The root-local Generator shall remain a compiler-private `i32`
ownership token. LLVM shall derive placement, alignment, and call lowering
from the target triple and data layout; the backend shall hard-code no AMD64
register convention. Two aligned debug-only Result pointer shadows shall
preserve the yielded action value and captured initial lifetime. DWARF and GDB
shall expose the complete recursive Generator classifier and value, the Result
product classifier, ordered payload fields, both yielded/captured `(7,
"item")` values, ordered yield/action/resumption/final source locations, and
the Topal entry frame.

An Error value, another Result success type, product arity, field order, input,
action, final, direction, yield, or value; multiple yields; additional body
statements; close handling; nested construction; ordinary-function construction
or Generator parameter/result transfer outside
`TOPAL-COMP-GENERATOR-NESTED-FUNCTION-BOUNDARY-001`; repeated consumption; abandonment;
libraries; and external boundaries shall remain rejected before LLVM. The
lowering shall introduce no semantic Generator object or state allocation,
generic Result/product dispatcher, callback, indirect call, unwind dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library calling convention or Generator/Result/product ABI,
or `topal-native/6` revision. Result, product, Int, and String allocation,
equality, display, and Linux syscalls shall remain wholly owned by Topal.
Future compiled-library metadata shall encode recursive classifiers,
independent directions, nominal alternatives/error vocabularies, product
arity/order, field identities, success/error evidence, equality and ordered
yield/action/resume/final provenance, allocation-failure effects, declaration/
construction sites, captures/effects, ownership/consumption/close state,
native-representation identity, and target adapters rather than expose private
tokens, debug slots, object layouts, or checked-program nodes. This realizes
`TOPAL-COMPILER-GENERATOR-NESTED-RESULT-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`,
`TOPAL-TYPE-RESULT-001`, and `TOPAL-TYPE-PRODUCT-001` for compiler increment
5an.

## TOPAL-COMP-GENERATOR-FINAL-DECISION-001 — Post-resume final Boolean decision

The checked compiler shall admit a root custom generator with one named Boolean
initial parameter and directions `Generator Boolean Unit String`. Its body
shall consist only of one discarded `yield initial` followed by a complete
decision on `initial` whose `true` action is the exact String `"accepted"` and
whose `otherwise` action is the exact String `"rejected"`. The shared
regression shall start the generator with `true`; the fresh result shall be
bound and consumed exactly once by a foreach action consisting only of the
discarded `not value` for its named yielded parameter.

The checked program shall retain the independent Boolean yield and String
final directions, the initial-parameter suspension, action, Unit resumption,
decision subject and ordered alternatives/actions, declaration provenance, and
ownership edge. Application shall evaluate the initial Boolean once. Traversal
shall pass that same value to the action, resume with Unit, evaluate the
captured initial as the decision subject once, execute exactly the selected
String action, and return that String as the final/root value. LLVM O0 shall
retain a direct conditional branch, distinct true/false String blocks, and a
typed join without relying on folding, inlining, or dead-code elimination.

On Linux x86-64, the Boolean shall remain a private `i1`, each selected String
shall use the existing Topal-owned immutable descriptor, and the root-local
Generator shall remain a compiler-private `i32` ownership token. LLVM shall
derive placement, alignment, branch lowering, and calls from the target triple
and data layout; the backend shall hard-code no AMD64 register convention. An
aligned debug-only Boolean shadow shall preserve the yielded action value.
DWARF and GDB shall expose the complete Generator classifier and value, yielded
and captured Boolean values, ordered yield/action/decision source locations,
and the Topal entry frame.

Another input classifier, yield, action, decision subject, matcher order,
String action, direction, value, or final expression; multiple yields; additional body
statements; close handling; nested or ordinary-function construction;
Generator parameter/result transfer; repeated consumption; abandonment;
libraries; and external boundaries shall remain rejected before LLVM. The
lowering shall introduce no semantic Generator object or state allocation,
generic decision or Generator dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator/String ABI,
or `topal-native/6` revision. Boolean decisions, String construction/display,
and Linux syscalls shall remain wholly owned by Topal. Future compiled-library
metadata shall encode independent directions, ordered decision matchers/
actions, subject/final provenance, declaration/construction sites, captures/
effects, ownership/consumption/close state, native-representation identity,
and target adapters rather than expose private tokens, debug slots, LLVM
blocks, object layouts, or checked-program nodes. This realizes
`TOPAL-COMPILER-GENERATOR-FINAL-DECISION-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`,
`TOPAL-GENERATOR-FINAL-RETURN-001`, and `TOPAL-DECISION-BOOLEAN-001` for
compiler increment 5ap.

## TOPAL-COMP-GENERATOR-NESTED-NONE-001 — Absent recursive Optional directions

The checked compiler shall admit a root custom generator with one named
`Optional (Int, String)` initial parameter and directions
`Generator Optional (Int, String) Unit Optional (Int, String)`. Its body shall
consist only of one discarded `yield initial` followed by
`None (Int, String)`. The shared regression shall start the generator with
exact `None (Int, String)`; the fresh result shall be bound and consumed exactly
once by a foreach action consisting only of the discarded
`candidate = (None (Int, String))` for its named yielded parameter.

The checked program shall retain the nominal Optional identity, None
alternative, positional-product arity and order, and Int and String field
identities separately in all three Generator directions despite the absence of
a payload. It shall also retain the initial-parameter yield and suspension,
tag-equality action, Unit resumption, distinct final None construction,
declaration provenance, and ownership edge. Application shall construct the
initial absent Optional once. Traversal shall pass that same immutable value to
the action, construct the absent action operand once, compare tags before any
payload observation, resume with Unit, and only then construct the final absent
Optional. This order shall hold at LLVM O0 without relying on folding, inlining,
or dead-code elimination.

On Linux x86-64, each admitted None shall use the existing Topal-owned Optional
tagged pointer header with no product-payload allocation. The root-local
Generator shall remain a compiler-private `i32` ownership token. LLVM shall
derive placement, alignment, and call lowering from the target triple and data
layout; the backend shall hard-code no AMD64 register convention. Two aligned
debug-only Optional pointer shadows shall preserve the yielded action value and
captured initial lifetime. DWARF and GDB shall expose the complete recursive
Generator classifier and value, the Optional product classifier and ordered
payload field types, both yielded/captured None values, ordered yield/action/
resumption/final source locations, and the Topal entry frame.

A Some value, mixed Some/None graph, another Optional payload, product arity,
field order, input, action, final, direction, yield, or value; multiple yields;
additional body statements; close handling; nested or ordinary-function
construction; Generator parameter/result transfer; repeated consumption;
abandonment; libraries; and external boundaries shall remain rejected before
LLVM. The lowering shall introduce no semantic Generator object or state
allocation, generic Optional/product dispatcher, callback, indirect call,
unwind dependency, C/C++ runtime, other-language standard library, needed
library, dynamic relocation, public/library calling convention or Generator/
Optional/product ABI, or `topal-native/6` revision. Optional tag construction/
equality, display, and Linux syscalls shall remain wholly owned by Topal.
Future compiled-library metadata shall encode recursive classifiers,
independent directions, nominal alternatives, product arity/order and field
identities, absence evidence, equality and ordered yield/action/resume/final
provenance, allocation-failure effects, declaration/construction sites,
captures/effects, ownership/consumption/close state, native-representation
identity, and target adapters rather than expose private tokens, debug slots,
object layouts, or checked-program nodes. This realizes
`TOPAL-COMPILER-GENERATOR-NESTED-NONE-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`,
`TOPAL-TYPE-OPTIONAL-CONSTRUCT-001`, and `TOPAL-TYPE-PRODUCT-001` for compiler
increment 5ao.

## TOPAL-COMP-GENERATOR-RECURSIVE-NOMINAL-001 — Recursive nominal value directions

The checked compiler shall admit the declaration
`Choice is Enum (First, Second)` and a root custom generator with one named
`(Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))`
initial parameter and the same yield and result directions with Unit
resumption. Its body shall consist only of one discarded `yield initial`
followed by `(Some Second, Second)`. The shared regression shall start the
generator with exact `(Some First, First)`; the fresh result shall be bound and
consumed exactly once by a foreach action consisting only of the discarded
`candidate = (Some First, First)` for its named yielded parameter.

The checked program shall retain the nominal Choice identity and ordered First/
Second alternatives inside both recursive fields, the Optional and Result
identities, arithmetic error vocabulary, product arity/order, initial-parameter
yield and suspension, guarded structural-equality action, Unit resumption,
distinct final alternatives, declaration provenance, and ownership edge
separately in all three Generator directions. Application shall construct the
initial recursive product once. Traversal shall pass that same immutable value
to the action, separately construct its comparison operand, compare Optional
and Result tags before loading their boxed Choice tags, resume with Unit, and
only then construct the final recursive product. This order shall hold at LLVM
O0 without relying on folding, inlining, or dead-code elimination.

On Linux x86-64, each admitted Choice payload inside Optional or Result shall
use a Topal-owned aligned four-byte box composed with the existing tagged
pointer header; six boxes shall be constructed across input, action, and final
values. The product shall remain a private two-pointer aggregate and the root-
local Generator a compiler-private `i32` ownership token. LLVM shall derive
placement, alignment, and call lowering from the target triple and data layout;
the backend shall hard-code no AMD64 register convention. Two aligned debug-
only aggregate shadows shall preserve the yielded action value and captured
initial lifetime. DWARF and GDB shall expose the complete recursive Generator
classifier and value, ordered product members, Optional and Result classifiers,
the nested Choice identity and alternatives, yielded/captured values, ordered
yield/action/resumption/final source locations, and the Topal entry frame.

A None or error value, mixed alternative, another Enum declaration or order,
recursive field classifier, product arity/order, error vocabulary, input,
action, final, direction, yield, or value; multiple yields; additional body
statements; close handling; nested or ordinary-function construction;
Generator parameter/result transfer; repeated consumption; abandonment;
libraries; and external boundaries shall remain rejected before LLVM. The
lowering shall introduce no semantic Generator object or state allocation,
generic recursive/Optional/Result dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator/product/
Optional/Result/Choice ABI, or `topal-native/6` revision. Enum boxing, Optional
and Result construction/equality, display, and Linux syscalls shall remain
wholly owned by Topal. Future compiled-library metadata shall encode recursive
nominal identity, ordered alternatives, error vocabularies, product arity/order
and field identities, independent directions, success/absence evidence,
equality and ordered yield/action/resume/final provenance, allocation-failure
effects, declaration/construction sites, captures/effects, ownership/
consumption/close state, native-representation identity, and target adapters
rather than expose private tokens, debug slots, object layouts, or checked-
program nodes. This realizes
`TOPAL-COMPILER-GENERATOR-RECURSIVE-NOMINAL-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`,
`TOPAL-TYPE-ENUM-001`, `TOPAL-TYPE-OPTIONAL-CONSTRUCT-001`,
`TOPAL-TYPE-RESULT-001`, and `TOPAL-TYPE-PRODUCT-001` for compiler increment
5aq.

## TOPAL-COMP-GENERATOR-LOCAL-FUNCTION-001 — Retained local declarations

The checked compiler shall admit the unchanged
`examples/language/custom-generator-local-function.t` regression: an exact
root `Generator Boolean Unit String` whose body declares local
`Choice is Enum (Accepted, Rejected)`, declares local
`label (value : Choice) -> String` with the complete accepted/rejected String
mapping, yields its named Boolean initial parameter once, and calls
`label Accepted` after Unit resumption. The fresh generator shall be consumed
exactly once by the discarded `not value` foreach action.

The checked model shall retain the local nominal identity and ordered
alternatives, local function signature and Enum decision, initial-parameter
yield, post-resume direct call and argument, source provenance, and ownership
edge separately. The local names shall be unavailable to the root and consumer
environments. Application shall evaluate the initial Boolean once. Traversal
shall run the action, resume with Unit, directly invoke the retained function,
and use its String result only afterward. The O0 IR shall preserve this order
without relying on an LLVM optimization.

The Linux x86-64 backend shall use the existing private `i32` representations
for Choice and the root-local Generator token. It shall emit `label` as an
internal, non-inlined `fastcc` definition with a direct call and no closure or
environment object. LLVM shall own target argument, return, stack, and register
placement. DWARF and GDB shall expose the complete Generator classifier and
value, yielded and captured Booleans, ordered yield/action/resume/call source
locations, the local function subprogram, its nominal Choice parameter and
Accepted value, and both the local-function and Topal entry frames.

Another local declaration, enum name/alternative/order, function signature or
body, capture, final call or argument, generator direction, action, yield,
result or ownership path; close handling; parameter/result transfer; repeated
consumption; abandonment; libraries; and external boundaries shall remain
rejected before LLVM. This shall add no semantic Generator object or state
allocation, closure environment, dispatcher, callback, indirect call, unwind
dependency, foreign runtime, C/C++ runtime, other-language standard library,
needed library, dynamic relocation, public/library calling convention,
Generator/function/Choice ABI, or `topal-native/6` revision. Future
compiled-library metadata shall encode lexical declaration identity and parent
scope, nominal alternatives, function signature and checked body graph,
capture set, suspension reachability, direct-call and ordered yield/action/
resume/final provenance, effects, ownership/consumption/close state,
native-representation identity, and target adapters rather than expose private
symbols, tags, tokens, debug slots, object layouts, or checked-program nodes.
This realizes `TOPAL-COMPILER-GENERATOR-LOCAL-FUNCTION-001`,
`TOPAL-GENERATOR-LOCAL-FUNCTION-001`, `TOPAL-GENERATOR-LOCAL-ENUM-001`,
`TOPAL-GENERATOR-SUSPEND-001`, `TOPAL-FUNCTION-ORDINARY-001`, and
`TOPAL-TYPE-ENUM-001` for compiler increment 5ar.

## TOPAL-COMP-GENERATOR-LOCAL-CLOSE-001 — Restored declarations on close

The checked compiler shall admit the unchanged
`examples/language/custom-generator-local-close-handler.t` regression: an
exact root `Generator Character Unit Unit` whose body declares local
`CloseChoice is Enum (Closed, Continued)`, declares the local ordinary
`cleanup (choice : CloseChoice) -> Unit`, binds one `yield initial` Result, and
handles its qualified `generator-closed`, fallback Error, and Ok alternatives.
An ordinary `abandon` function shall construct one fresh instance and close it
by reaching function-scope Unit without consuming the yielded Character.

The checked model shall retain the local nominal identity and alternatives,
local function signature and Unit body, yield-result decision and ordered
branches, direct cleanup calls and arguments, declaration and call provenance,
and ownership/close edge separately. Close delivery shall restore the same
generator-local declaration state that was active at suspension, select only
the qualified close branch, directly call `cleanup Closed`, and complete the
generator before `abandon` returns. The local names shall remain unavailable
to the root and consumer environments. This O0 order shall not rely on an LLVM
optimization.

On Linux x86-64, CloseChoice and the generator ownership token shall use
compiler-private `i32` representations. The backend shall emit `cleanup` as an
internal, non-inlined `fastcc` definition and invoke it directly after the
Topal-owned Result payload and code selection. A debugger-only aligned shadow
shall preserve the otherwise-unused enum parameter. LLVM shall derive machine
argument, return, stack, alignment, and register placement from the target
triple and data layout. DWARF and GDB shall expose the close Result and code,
the local function subprogram, its complete nominal CloseChoice parameter and
Closed value, ordered close/call locations, and the cleanup, abandon, and
Topal entry frames.

Another local declaration, enum name/alternative/order, function signature or
body, capture, yield, decision subject/branch/order/action, cleanup call or
argument, generator direction, result or ownership path; successful traversal,
multiple yields or owned generators; parameter/result transfer; libraries; and
external boundaries shall remain rejected before LLVM. This shall add no
semantic Generator object or state allocation, closure environment, close
dispatcher, callback, indirect call, unwind dependency, foreign runtime,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library calling convention, Generator/function/Choice ABI,
or `topal-native/6` revision. Future compiled-library metadata shall encode
lexical declaration identity and parent scope, nominal alternatives, function
signature and checked body graph, capture set, suspension and close
reachability, ordered Result matchers and fallback, direct-call and close-site
provenance, lexical domain, effects, ownership/consumption state,
native-representation identity, and target adapters rather than expose private
symbols, tags, tokens, debug slots, object layouts, or checked-program nodes.
This realizes `TOPAL-COMPILER-GENERATOR-LOCAL-CLOSE-001`,
`TOPAL-GENERATOR-LOCAL-FUNCTION-001`, `TOPAL-GENERATOR-LOCAL-ENUM-001`,
`TOPAL-GENERATOR-CLOSE-001`, `TOPAL-GENERATOR-CLOSE-HANDLER-001`,
`TOPAL-GENERATOR-ERROR-CODE-001`, `TOPAL-FUNCTION-ORDINARY-001`, and
`TOPAL-TYPE-ENUM-001` for compiler increment 5as.

## TOPAL-COMP-GENERATOR-OVERLOAD-001 — Ordered inputs and typed traversal results

The checked compiler shall admit the unchanged
`examples/language/custom-generator-overloads.t` regression. It declares, in
source order, one `select (Int)` overload yielding Int and one
`select (Int, String)` overload yielding String; both resume with Unit and
return String. Application shall evaluate its argument once, flatten an
unlabeled positional product for the binary candidate, select the first
complete ordered classifier match, and bind the selected operands in
declaration order. The admitted calls shall use exact `7` and `(7, "item")`
inputs respectively. A duplicate ordered input signature shall be rejected even
when its yield, resume, result, or body differs.

The checked model shall retain each overload as a distinct declaration with
ordered input classifiers, parameters, captured values, yield/result
directions, body graph, declaration provenance, and ownership edge. The unary
traversal shall yield its Int input, run the exact increment action, resume
with Unit, and bind final String `"unary"`. The binary traversal shall observe
its Int prefix before yielding the captured suffix, run the exact String
emptiness action, resume with Unit, and bind final String `"binary"`. Each
typed foreach result binding shall be a fresh ordinary root binding available
only after traversal completion. All ordering shall hold at LLVM O0 without
folding, inlining, or dead-code elimination.

On Linux x86-64, the overload set and captured input vectors shall remain
checked compile-session data. Each root-local Generator shall use the existing
private `i32` ownership token, while Int and String retain their Topal-owned
pointer representations. LLVM shall derive physical placement and alignment
from the target triple and data layout; no AMD64 register convention shall be
hard-coded. Target-aligned debugger-only input and action shadows shall expose
both Generator classifiers, ordered binary inputs, yielded values, typed final
bindings, source locations, and the Topal entry frame in DWARF/GDB.

Other arities, classifiers, input values, bodies, directions, argument packaging, actions,
results, nested/function construction, parameter/result transfer, repeated
consumption, abandonment, close handling, libraries, and external boundaries
shall remain rejected before LLVM. This shall add no semantic Generator object
or state allocation, overload dispatcher, callback, indirect call, unwind
dependency, foreign runtime, C/C++ runtime, other-language standard library,
needed library, dynamic relocation, public/library calling convention,
Generator ABI, or `topal-native/6` revision. Future compiled-library metadata
shall encode declaration order, complete ordered input classifiers, packaging,
independent directions, body and suspension graphs, captures, effects,
ownership/consumption/close state, typed final-result provenance,
native-representation identity, and target adapters rather than private tokens,
debug slots, pointer layouts, or checked-program nodes. This realizes
`TOPAL-COMPILER-GENERATOR-OVERLOAD-001`, `TOPAL-GENERATOR-OVERLOAD-001`,
`TOPAL-GENERATOR-FOREACH-RESULT-001`, `TOPAL-GENERATOR-FINAL-RETURN-001`, and
`TOPAL-TYPE-PRODUCT-001` for compiler increment 5at.

## TOPAL-COMP-GENERATOR-FUNCTION-BOUNDARY-001 — Specialized scalar continuation transfer

The checked compiler shall admit the unchanged
`examples/language/custom-generator-generic-function-boundaries.t` regression.
Its exact `numbers` declaration shall be `Generator Int Unit String`, yield its
Int input once, resume with Unit, and return String `"done"`. The ordinary
single-Int `make` function shall return a fresh continuation without closing
it. The ordinary single-Generator `consume` function shall receive sole
ownership, traverse it once with the exact discarded `value + 1` action, bind
the final String, and return it. The root graph shall pass exact Int `7` through
`make`, bind the returned continuation, transfer it to `consume`, and print
`"done"`.

The checked model shall preserve the complete Generator classifier,
declaration, parameter-derived construction, exact call-specialized input,
single suspension, Unit resumption, final-result graph, function-result edge,
function-parameter edge, and once-only consumption as separate facts. The
returned-provenance specialization shall replace the factory-local input name
with the already evaluated exact caller operand before checking the consumer;
it shall not reevaluate the source call. Factory exit and consumer entry shall
not deliver close. Consumer action, resumption, final String construction, and
return shall remain ordered at LLVM O0 without relying on inlining, constant
folding, or dead-code elimination.

On Linux x86-64, the private factory shall lower as one Topal Int pointer to an
`i32` ownership token, and the private consumer as that token to one Topal
String pointer. LLVM `fastcc`, the target triple, and target data layout shall
select physical placement and alignment; no AMD64 register or return placement
shall be hard-coded. Target-aligned debug-only shadows shall keep the factory
Int, complete Generator consumer parameter, yielded Int action value, final
String binding, ordinary-function frames, and Topal entry frame visible in
DWARF/GDB.

Other Generator classifiers, inputs, declaration/body shapes, factory or
consumer forms, actions, results, arities, recursion, nesting, repeated use,
abandonment, close handling, packages, libraries, and external boundaries
shall remain rejected before LLVM. This shall add no semantic Generator object
or state allocation, runtime transfer/traversal dispatcher, callback, indirect
call, unwind dependency, foreign runtime, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library calling
convention, Generator ABI, or `topal-native/6` revision. Future compiled-library
metadata shall encode complete classifiers and directions, declaration and
body/suspension graphs, construction and transfer sites, exact or symbolic
captures, effects, ownership/consumption/close state, final-result provenance,
native-representation identity, and target adapters rather than private
specializations, tokens, debug slots, pointer layouts, or checked-program
nodes. This realizes `TOPAL-COMPILER-GENERATOR-FUNCTION-BOUNDARY-001`,
`TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001`,
`TOPAL-GENERATOR-FUNCTION-RESULT-001`, and
`TOPAL-GENERATOR-FUNCTION-PARAMETER-001` for compiler increment 5au.

## TOPAL-COMP-GENERATOR-COMPOUND-FUNCTION-BOUNDARY-001 — Specialized positional-product continuation transfer

The checked compiler shall admit the unchanged
`examples/language/custom-generator-compound-function-boundaries.t` regression.
Its exact `pairs` declaration shall be
`Generator (Int, String) Unit (Int, String)`, yield its positional-product input
once, resume with Unit, and return `(8, "done")`. The ordinary `make` function
shall accept `(Int, String)` and return the fresh continuation without closing
it. The ordinary `consume` function shall receive sole ownership, traverse it
once with the exact discarded `value = (7, "item")` action, bind the final
product, and return it. The root graph shall pass exact `(7, "item")` through
`make`, bind the continuation, transfer it to `consume`, and print
`(8, "done")`.

The checked model shall preserve product arity, field order and classifiers in
the initial, yield, and final directions together with declaration,
parameter-derived construction, exact call-specialized input, suspension,
field-wise action, Unit resumption, final-result graph, both function-transfer
edges, and once-only consumption. The factory-local input name shall be
specialized to the already evaluated caller aggregate without reevaluation.
Factory exit and consumer entry shall not deliver close. Ordered Int equality,
String equality, final-product construction, and return shall hold at LLVM O0
without inlining, folding, or dead-code elimination.

On Linux x86-64, the private factory shall lower from the existing LLVM
aggregate of Topal-owned Int and String pointers to an `i32` ownership token;
the private consumer shall lower from that token to the aggregate. LLVM
`fastcc`, the target triple, and target data layout shall determine aggregate
placement, alignment, and return lowering; no AMD64 register or return
convention shall be hard-coded. Target-aligned debug shadows shall expose the
factory product, complete Generator parameter, yielded product, final product,
ordered `_0` and `_1` fields, ordinary-function frames, and Topal entry frame
in DWARF/GDB.

Other product shapes, field classifiers or order, inputs, declarations,
factories, consumers, actions, results, arities, recursion, nesting, repeated
use, abandonment, close handling, packages, libraries, and external boundaries
shall remain rejected before LLVM. This shall add no semantic Generator object
or state allocation, transfer/traversal dispatcher, callback, indirect call,
unwind dependency, foreign runtime, C/C++ runtime, other-language standard
library, needed library, dynamic relocation, public/library calling convention,
Generator/product ABI, or `topal-native/6` revision. Future compiled-library
metadata shall encode canonical product arity, order, field identities,
complete Generator directions, declaration/body/suspension graphs,
construction and transfer sites, exact or symbolic captures, field-wise
operations, effects, ownership/consumption/close state, final-result provenance,
native-representation identity, and target adapters rather than private
specializations, tokens, debug slots, aggregate layouts, or checked-program
nodes. This realizes
`TOPAL-COMPILER-GENERATOR-COMPOUND-FUNCTION-BOUNDARY-001`,
`TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001`,
`TOPAL-GENERATOR-FUNCTION-RESULT-001`,
`TOPAL-GENERATOR-FUNCTION-PARAMETER-001`,
`TOPAL-GENERATOR-FOREACH-RESULT-001`, and `TOPAL-TYPE-PRODUCT-001` for compiler
increment 5av.

## TOPAL-COMP-GENERATOR-NESTED-FUNCTION-BOUNDARY-001 — Specialized recursive-value continuation transfer

The checked compiler shall admit the unchanged
`examples/language/custom-generator-nested-function-boundaries.t` regression.
Its exact `pairs` declaration shall be
`Generator Optional (Int, String) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)`.
It shall yield its initial Some product once, resume with Unit, and return the
successful product `(8, "done")`. The ordinary `make` function shall accept
exact `Optional (Int, String)` and return the fresh continuation without
closing it. The ordinary `consume` function shall receive sole ownership,
traverse it with the exact discarded `value = (Some (7, "item"))` action, bind
the final Result, and return it. The root graph shall pass exact
`Some (7, "item")` through both transfer edges and print `(8, "done")`.

The checked model shall preserve the nominal Optional and Result identities,
arithmetic-error vocabulary, success evidence, positional-product arity and
field order, and independent Generator directions. It shall separately retain
declaration, parameter-derived construction, exact call-specialized input,
suspension, tag-gated and field-wise action, Unit resumption, final-result
graph, both function-transfer edges, and once-only consumption. The
factory-local input shall be replaced by the already evaluated caller Optional
without reevaluation. Factory exit and consumer entry shall not deliver close.
Optional tag checks, Int/String field equality, successful Result construction,
and return shall remain ordered at LLVM O0 without relying on inlining,
folding, or dead-code elimination.

On Linux x86-64, the private factory shall lower from the existing Topal-owned
Optional pointer to an `i32` ownership token, and the private consumer shall
lower from that token to the existing Topal-owned Result pointer. Product
payloads shall retain their Topal-owned aligned Int/String pointer storage.
LLVM `fastcc`, the target triple, and target data layout shall determine
placement and alignment; no AMD64 register or return convention shall be
hard-coded. A target-aligned factory parameter shadow shall retain the
otherwise compile-time-only input lifetime. DWARF/GDB shall expose the complete
Generator classifier, Optional and Result product classifiers, error
vocabulary, initial/yield/final values, ordinary-function frames, and Topal
entry frame.

Other nested classifiers, alternatives, product shapes, inputs, declarations,
factories, consumers, actions, results, arities, recursion, nesting, repeated
use, abandonment, close handling, packages, libraries, and external boundaries
shall remain rejected before LLVM. This shall add no semantic Generator object
or state allocation, transfer/traversal dispatcher, callback, indirect call,
unwind dependency, foreign runtime, C/C++ runtime, other-language standard
library, needed library, dynamic relocation, public/library calling convention,
Generator/Optional/Result/product ABI, or `topal-native/6` revision. Future
compiled-library metadata shall encode nominal wrapper identities and
alternatives, error vocabulary, recursive product arity/order/field identities,
complete Generator directions, declaration/body/suspension graphs,
construction and transfer sites, exact or symbolic captures, tag and
field-operation provenance, effects, ownership/consumption/close state,
final-result evidence, native-representation identity, and target adapters
rather than private specializations, tokens, debug slots, object layouts, or
checked-program nodes. This realizes
`TOPAL-COMPILER-GENERATOR-NESTED-FUNCTION-BOUNDARY-001`,
`TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001`,
`TOPAL-GENERATOR-FUNCTION-RESULT-001`,
`TOPAL-GENERATOR-FUNCTION-PARAMETER-001`,
`TOPAL-GENERATOR-FOREACH-RESULT-001`, `TOPAL-TYPE-OPTIONAL-CONSTRUCT-001`,
`TOPAL-TYPE-RESULT-001`, and `TOPAL-TYPE-PRODUCT-001` for compiler increment
5aw.

## TOPAL-COMP-GENERATOR-LIST-FUNCTION-BOUNDARY-001 — Specialized List continuation transfer

The checked compiler shall admit the unchanged
`examples/language/custom-generator-list-values.t` regression. Its exact
`relay` declaration shall be `Generator List Int Unit List Int`, yield its
initial List once, resume with Unit, and return `initial append 9`. The ordinary
`make` function shall accept exact `List Int` and return the fresh continuation
without closing it. The ordinary `consume` function shall receive sole
ownership, traverse it with the exact discarded `entry-count values` action,
bind the final List, and return it. The root graph shall pass exact `one 7`
through both transfer edges and print
`Entry ( 7, Entry ( 9, Empty ) )`.

The checked model shall preserve recursive List identity, the exact Int element
classifier, immutable entry order, independent Generator directions,
declaration, parameter-derived construction, exact call-specialized input,
suspension, action, final append graph, both function-transfer edges, and once-
only consumption. The factory-local input shall be replaced by the already
evaluated caller List without reevaluation. Factory exit and consumer entry
shall not deliver close. The entry-count action shall complete before Unit
resumption, followed in order by final singleton allocation, immutable append,
and return at LLVM O0 without relying on inlining, folding, or dead-code
elimination.

On Linux x86-64, the private factory shall lower from the existing Topal-owned
`List Int` pointer to an `i32` ownership token, and the private consumer shall
lower from that token to the existing List pointer. LLVM `fastcc`, the target
triple, and target data layout shall determine physical placement and
alignment; no AMD64 register or return convention shall be hard-coded. A
target-aligned factory parameter shadow and List action/final-lifetime shadows
shall retain otherwise compile-time-only values. DWARF/GDB shall expose the
complete Generator classifier, initial, yielded and final Lists, ordinary-
function frames, and Topal entry frame.

Other List element classifiers, inputs, declarations, factories, consumers,
actions, final operations or values, yields, directions, arities, recursion,
nesting, repeated use, abandonment, close handling, packages, libraries, and
external boundaries shall remain rejected before LLVM. This shall add no
semantic Generator object/state allocation, transfer/traversal dispatcher,
callback, indirect call, unwind dependency, foreign runtime, C/C++ runtime,
other-language standard library, foreign allocator, needed library, dynamic
relocation, public/library calling convention, Generator/List ABI, or
`topal-native/6` revision. Future compiled-library metadata shall encode
recursive nominal List identity, element classifier, immutable operation and
allocation effects, complete Generator directions, declaration/body/suspension
graph, construction and transfer sites, exact or symbolic captures,
action/final-operation provenance, ownership/consumption/close state, native-
representation identity, and target adapters rather than private
specializations, tokens, node pointers/layouts, debug slots, or checked-program
nodes. This realizes
`TOPAL-COMPILER-GENERATOR-LIST-FUNCTION-BOUNDARY-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`,
`TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001`,
`TOPAL-GENERATOR-FUNCTION-RESULT-001`,
`TOPAL-GENERATOR-FUNCTION-PARAMETER-001`,
`TOPAL-GENERATOR-FOREACH-RESULT-001`, `TOPAL-TYPE-LIST-CONSTRUCT-001`,
`TOPAL-LIST-APPEND-001`, and `TOPAL-LIST-ENTRY-COUNT-001` for compiler increment
5ax.

## TOPAL-COMP-GENERATOR-PRODUCT-001 — Exact positional-product generator directions

The checked compiler shall admit a root custom generator with one named
`(Int, String)` initial parameter and directions
`Generator (Int, String) Unit (Int, String)`. Its body shall consist only of one
discarded `yield initial` followed by `(8, "done")` as its final expression. The
shared regression starts the generator with `(7, "item")`; the fresh result
shall be bound and consumed exactly once by a foreach action consisting only of
the discarded `value = (7, "item")` for its named yielded parameter.

The checked program shall retain positional-product arity, source field order,
the Int and String field classifiers separately in all three Generator
directions, the initial-parameter yield and suspension, field-wise equality
action, Unit resumption, distinct final product, declaration provenance, and
ownership edge. Application shall evaluate both initial fields once from left
to right. Traversal shall pass that same immutable product to the action,
compare its Int and String fields in order, resume with Unit, and only then
materialize `(8, "done")` as the final product. This order shall hold with LLVM
optimization disabled and shall not depend on folding, inlining, or dead-code
elimination.

On Linux x86-64, the product shall use the existing compiler-private LLVM
aggregate of Topal-owned Int and String pointers, and the root-local Generator
shall remain a compiler-private `i32` ownership token. LLVM shall derive
aggregate placement, alignment, and call lowering from the target triple and
data layout; the backend shall hard-code no AMD64 register convention. Two
aligned debug-only product shadows and four lifetime/source anchor stores shall
keep the yielded action value and captured initial inspectable. DWARF and GDB
shall expose the complete
`Generator (Int, String) Unit (Int, String)` classifier and value, ordered `_0`
and `_1` product members, the yielded `(7, "item")`, captured initial
`(7, "item")`, ordered yield/action/resumption/final source locations, and the
Topal entry frame.

Another product arity, field classifier, order, literal, or direction; literal
or multiple yields; a final expression or action other than the exact products
above; labeled products; additional body statements; close handling; nested
construction; ordinary-function construction or Generator parameter/result
transfer outside `TOPAL-COMP-GENERATOR-COMPOUND-FUNCTION-BOUNDARY-001`; repeated
consumption; abandonment; libraries; and external boundaries shall remain
rejected before LLVM. The lowering shall introduce no semantic Generator
object or state allocation, dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator/product
ABI, or `topal-native/6` revision. Product equality/display, Int/String storage,
and Linux syscalls shall remain wholly owned by Topal. Future compiled-library
metadata shall encode independent initial and direction classifiers,
positional-product arity and ordered field identities, field equality, ordered
yield/action/resume/final provenance, allocation-failure effects, declaration
and construction sites, captures/effects, ownership/consumption/close state,
native-representation identity, and target adapters rather than expose the
private token, debug slots, aggregate/pointer layouts, or checked-program node
layout. This realizes `TOPAL-COMPILER-GENERATOR-PRODUCT-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-TYPE-PRODUCT-001` for compiler increment 5aj.

## TOPAL-COMP-GENERATOR-ENUM-001 — Exact nominal Enum generator directions

The checked compiler shall admit the exact prior declaration
`Choice is Enum (First, Second)` followed by a root custom generator with one
named `Choice` initial parameter and directions
`Generator Choice Unit Choice`. Its body shall consist only of one discarded
`yield initial` followed by `Second` as its final expression. The shared
regression starts the generator with `First`; the fresh result shall be bound
and consumed exactly once by a foreach action consisting only of the discarded
`choice = First` for its named yielded parameter.

The checked program shall retain the nominal Choice identity, ordered First and
Second alternatives and private tags separately from all three Generator
directions, the initial-parameter yield and suspension, exact equality action,
Unit resumption, distinct final alternative, declaration provenance, and
ownership edge. Application shall evaluate its initial expression exactly once.
Traversal shall pass that same immutable Choice value to the action, compare it
with First once, resume with Unit, and only then produce Second as the final
Choice value. This order shall hold with LLVM optimization disabled and shall
not depend on folding, inlining, or dead-code elimination.

On Linux x86-64, Choice shall retain its existing compiler-private `i32` tag
representation, and the root-local Generator shall remain a compiler-private
`i32` ownership token. LLVM shall select placement and call lowering from the
target triple and data layout; the backend shall hard-code no AMD64 register
convention. Two aligned debug-only `i32` shadow slots and four lifetime/source
anchor stores shall keep the yielded action value and captured initial
inspectable even when LLVM folds the constant equality and final tag selection.
DWARF and GDB shall expose the complete `Generator Choice Unit Choice`
classifier and value, the Choice alternatives, yielded First, captured initial
First, ordered yield/action/resumption/final source locations, and the Topal
entry frame.

Another Enum declaration, alternatives, order, or direction; literal or
multiple yields; a final expression or action other than the exact alternatives
above; additional body statements; close handling; nested or ordinary-function
construction; Generator parameter/result transfer; repeated consumption;
abandonment; libraries; and external boundaries shall remain rejected before
LLVM. The lowering shall introduce no semantic Generator object or state
allocation, dispatcher, callback, indirect call, unwind dependency, C/C++
runtime, other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or `topal-native/6`
revision. Enum comparison, display, and Linux syscalls shall remain wholly
owned by Topal. Future compiled-library metadata shall encode independent
initial and direction classifiers, nominal Enum identity, ordered alternative
identities and tags, equality operands, ordered yield/action/resume/final
provenance, declaration/construction sites, captures/effects, ownership/
consumption/close state, native-representation identity, and target adapters
rather than expose the private token, debug slots, tag layout, or checked-
program node layout. This realizes `TOPAL-COMPILER-GENERATOR-ENUM-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-TYPE-ENUM-001` for compiler increment 5ai.

## TOPAL-COMP-GENERATOR-NAT-001 — Exact Nat generator directions

The checked compiler shall admit a root custom generator with one named `Nat`
initial parameter and directions `Generator Nat Unit Nat` whose body consists
only of one discarded `yield initial` followed by `initial + 1` as its final
expression. One currently admitted `Nat` expression shall start the generator;
the shared regression uses the statically proven construction `Nat 7`. The
fresh result shall be bound and consumed exactly once by a foreach action
consisting only of the discarded `value + 1` for its named yielded parameter.

The checked program shall retain the Nat refinement identity separately from
its underlying exact Int representation and from all three Generator
directions, the proof-bearing input construction, initial-parameter yield and
suspension, exact action and final additions, Unit resumption, and ownership
edge. Application shall evaluate its initial expression exactly once. For the
shared closed input, construction shall preserve the nonnegative proof without
a runtime validation call. Traversal shall pass that same immutable Nat value
to the action, add exact one once, resume with Unit, and only then add exact one
to the captured initial for the final Nat value `8`. This order shall hold with
LLVM optimization disabled and shall not depend on folding, inlining, or dead-
code elimination.

On Linux x86-64, Nat shall retain its existing compiler-private representation
as an aligned pointer to Topal-owned arbitrary-precision Int storage; the
nonnegative constraint shall not introduce a native unsigned-integer ABI. The
root-local Generator shall remain a compiler-private `i32` ownership token.
LLVM shall select placement and call lowering from the target triple and data
layout; the backend shall hard-code no AMD64 register convention. Two aligned
debug-only pointer shadows shall preserve the yielded action value and captured
initial lifetime. DWARF and GDB shall expose the complete
`Generator Nat Unit Nat` classifier and value, the yielded Nat `7`, the
captured initial Nat `7`, ordered yield/action/resumption/final source
locations, and the Topal entry frame.

Literal or multiple yields; a final expression or action other than the exact
increments; additional body statements; another input, yield, resume, or final
classifier; general Nat arithmetic outside this exact graph; close handling;
nested or ordinary-function construction; Generator parameter/result transfer;
repeated consumption; abandonment; libraries; and external boundaries shall
remain rejected before LLVM. The lowering shall introduce no semantic Generator
object or state allocation, dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator ABI,
unsigned machine arithmetic, or `topal-native/6` revision. Exact integer
storage, allocation, addition, display, and Linux syscalls shall remain wholly
owned by Topal. Future compiled-library metadata shall encode independent
initial and direction classifiers, Nat refinement/proof and underlying Int
identities, validation provenance, exact addition operands, ordered yield/
action/resume/final provenance, allocation-failure effects, declaration/
construction sites, captures/effects, ownership/consumption/close state,
native-representation identity, and target adapters rather than expose the
private token, debug slots, Int object layout, or checked-program node layout.
This realizes `TOPAL-COMPILER-GENERATOR-NAT-001`,
`TOPAL-GENERATOR-DECLARATION-001`, `TOPAL-GENERATOR-SUSPEND-001`, and
`TOPAL-NUM-NAT-CONSTRUCT-001` for compiler increment 5ah.

## TOPAL-COMP-GENERATOR-RANGE-001 — Exact Range generator directions

The checked compiler shall admit a root custom generator with one named
`Range Int` initial parameter and directions
`Generator Range Int Unit Range Int` whose body consists only of one discarded
`yield initial` followed by `initial and (5 ..= 15)` as its final expression.
One currently admitted `Range Int` expression shall start the generator; the
shared regression uses `0 ..= 10`. The fresh result shall be bound and consumed
exactly once by a foreach action consisting only of the discarded membership
`5 in interval` for its named yielded parameter.

The checked program shall retain the Range classifier and nominal Int endpoint
classifier separately from all three Generator directions, the initial-
parameter yield and suspension, exact membership action, Unit resumption,
distinct final inclusive-bound construction and intersection, and ownership
edge. Application shall evaluate and construct its initial expression exactly
once. Traversal shall pass that same immutable Range value to the action,
evaluate membership once, resume the generator with Unit, and only then
construct `5 ..= 15` and intersect it with the initial value. The resulting
inclusive range shall be `5 ..= 10` for the shared source. This order shall
hold with LLVM optimization disabled and shall not depend on folding, inlining,
or dead-code elimination.

On Linux x86-64, the existing compiler-private Range representation shall
remain an aligned pointer to Topal-owned finite storage containing Int endpoint
pointers and inclusion flags. The root-local Generator shall remain a compiler-
private `i32` ownership token. LLVM shall select placement and call lowering
from the target triple and data layout; the backend shall hard-code no AMD64
register convention. Two aligned debug-only pointer shadows shall preserve the
yielded action value and captured initial lifetime. DWARF and GDB shall expose
the complete `Generator Range Int Unit Range Int` classifier and value, the
yielded `0 ..= 10`, the captured initial `0 ..= 10`, ordered yield/action/
resumption/final source locations, and the Topal entry frame.

Literal or multiple yields; a final expression other than the exact
intersection; additional body statements; another endpoint, action, input,
yield, resume, or final classifier; close handling; nested or ordinary-function
construction; Generator parameter/result transfer; repeated consumption;
abandonment; libraries; and external boundaries shall remain rejected before
LLVM. The lowering shall introduce no semantic Generator object or state
allocation, dispatcher, callback, indirect call, unwind dependency, C/C++
runtime, other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or `topal-native/6`
revision. Range/Int storage, allocation, membership, intersection, display, and
Linux syscalls shall remain wholly owned by Topal. Future compiled-library
metadata shall encode independent initial and direction classifiers, the
nominal endpoint identity, exact bounds, inclusivity, membership and
intersection operands, ordered yield/action/resume/final provenance,
allocation-failure effects, declaration/construction sites, captures/effects,
ownership/consumption/close state, native-representation identity, and target
adapters rather than expose the private token, debug slots, Range/Int object
layouts, or checked-program node layout. This realizes
`TOPAL-COMPILER-GENERATOR-RANGE-001`, `TOPAL-GENERATOR-DECLARATION-001`,
`TOPAL-GENERATOR-SUSPEND-001`, `TOPAL-RANGE-BOUNDS-001`, and
`TOPAL-RANGE-CLASSIFIER-001` for compiler increment 5ag.

## TOPAL-COMP-FUNCTION-001 — Scalar overloads and static functions

The compiler shall preserve source-ordered overload sets whose admitted
ordinary or static declarations have nullary, unary, or positional-product
scalar inputs. Declarations with identical input classifiers and staticness
shall be rejected regardless of parameter names or result classifiers. An
application shall analyze its arguments once and select the first header whose
input classifiers are statically proven applicable, including the admitted
lossless exact-number classifications and conversions; result context shall not
alter selection.

Every selected overload shall have a distinct checked call-graph identity,
private LLVM function, DWARF subprogram, parameters, and invocation-local
bindings. A static function body shall select only static callees, while root
and ordinary bodies may call ordinary or static declarations. Staticness shall
not create a public ABI distinction or permit compile-time execution to alter
observable behavior at O0.

Complete explicitly classified headers in one declaration scope shall be
collected before selected bodies are checked, permitting an admitted acyclic
body to call a later function declaration. Depth-first instantiation shall place
that callee before its caller in the checked program and emitted module while
ordinary value initializers remain source-ordered. DWARF and GDB shall retain
both source functions and their nested runtime frames.

These additions cover the admitted scalar cases of
`TOPAL-FUNCTION-STATIC-NULLARY-001`, `TOPAL-FUNCTION-STATIC-UNARY-001`,
`TOPAL-FUNCTION-STATIC-BINARY-001`, `TOPAL-FUNCTION-BLOCK-001`,
`TOPAL-FUNCTION-ORDINARY-001`, `TOPAL-FUNCTION-CALL-CHAIN-001`,
`TOPAL-FUNCTION-LOCAL-SCOPE-001`, `TOPAL-FUNCTION-OVERLOAD-001`,
and `TOPAL-FUNCTION-FORWARD-DECLARATION-001`. Dynamic structural applicability,
function values, nested functions, remaining recursion and its overload-identity
rules, and other function forms remain in later increment-3 dispositions.

## TOPAL-COMP-ROOT-NAMESPACE-001 — Direct executable-root qualification

For the admitted single-source application subset, the checked compiler model
shall recognize `root` as the executable root Scope value and shall resolve a
directly qualified root function from the collected root declarations before
ordinary source-ordered overload selection. The remaining operands shall be
analyzed once under the existing call rules, and a same-named lexical binding
shall not intercept the qualified root member. Binding or displaying `root`
shall not copy, flatten, or execute its declarations, and canonical display
shall match the interpreter's `<namespace root>` observation.

The backend shall render the sealed root identity through the Topal-owned Linux
writer and lower a qualified call directly to the selected private function.
DWARF/GDB shall retain the selected source function, typed argument, call site,
and runtime frame at O0. This shall require no runtime namespace lookup, Scope
object allocation, foreign dependency, C/C++ runtime, other-language standard
library, public tag ABI, or `topal-native/6` revision.

Qualified namespace-alias lookup, classified Scope bindings, root data-member
lookup, `use`, published interfaces, generators, package loading, and
compiled-library resolution remain rejected in this increment. This requirement
covers the direct-value and qualified-function subset of
`TOPAL-NAMESPACE-ROOT-001` and realizes
`TOPAL-COMPILER-ROOT-NAMESPACE-001` for compiler increment 6a.

## TOPAL-COMP-NAMESPACE-USE-001 — Static root namespace use

At source root, the checked compiler model shall accept `use root` and `use`
of an already retained root alias. It shall require the operand to be Scope,
preserve the same concrete root identity and source-position declaration
snapshot for optional binding, and diagnose a non-Scope operand with the shared
`E-USE-NON-NAMESPACE`. Subsequent qualified function or data selection shall
use the existing retained namespace facts without flattening them into lexical
lookup.

The backend shall erase the `use` operation itself. An observed binding shall
reuse the existing private Scope tag, display, and DWARF/GDB type, while
qualified functions remain direct private calls and qualified data retain their
original already-evaluated storage identity. Native tests shall cover the
existing interpreter regression, root and alias operands, non-Scope rejection,
exact output, checked snapshots, absent use/namespace runtime IR, freestanding
artifacts, source frames, Scope/value debugging, the shared corpus, and separate
resource baselines.

This shall add no lookup table, allocation, indirect dispatch, filesystem or
process-global resolution, foreign dependency, C/C++ runtime, other-language
standard library, public Scope ABI, or `topal-native/6` revision.
Multi-component or non-root published paths, nested namespaces, generator
members, function-body `use`, packages, source/compiled libraries, and public
interface metadata remain rejected. This realizes
`TOPAL-COMPILER-NAMESPACE-USE-001` and `TOPAL-NAMESPACE-USE-001` for compiler
increment 6b3a.

## TOPAL-COMP-NAMESPACE-FUNCTION-ALIAS-001 — Static function namespace aliases

For the admitted source-root subset, the checked compiler model shall retain an
immutable namespace snapshot when `root` or an existing alias is bound, with or
without an explicit `Scope` classifier. The snapshot shall preserve the root
identity, function declarations visible at the binding statement, complete
source-ordered overload sets, ordinary/static distinction, and the same facts
through a finite alias chain. Later declarations shall remain absent from an
earlier snapshot, and same-named caller bindings shall not intercept or combine
with qualified member selection.

Qualified function applications shall select from those captured declarations
and lower to the existing direct private LLVM call. The Scope value shall keep
its canonical display and target-exact DWARF enum identity, while captured
declarations remain checked frontend facts rather than runtime data. Native and
GDB tests shall cover alias observation, typed aliases, chaining, overload
selection, snapshot exclusion, exact output, and retained source call frames.

The implementation shall introduce no runtime namespace lookup, indirect call,
Scope allocation, foreign dependency, C/C++ runtime, other-language standard
library, public ABI, or `topal-native/6` revision. Data and generator members,
non-root alias bindings, general Scope function boundaries, `use`, packages,
and source/compiled libraries remain rejected. This requirement covers the
function-bearing subset of `TOPAL-NAMESPACE-ALIAS-001`,
`TOPAL-NAMESPACE-SNAPSHOT-001`, `TOPAL-NAMESPACE-OVERLOAD-001`,
`TOPAL-NAMESPACE-CLASSIFIER-001`, and `TOPAL-NAMESPACE-ALIAS-CHAIN-001`, and
realizes `TOPAL-COMPILER-NAMESPACE-FUNCTION-ALIAS-001` for increment 6b1.

## TOPAL-COMP-NAMESPACE-DATA-001 — Stable root data snapshots

The checked compiler model shall assign every admitted source-root binding a
stable internal storage identity distinct from its source/debug name. A binding
shall enter the live root data-member set only after its initializer has been
analyzed, and a root alias shall capture the data-member set visible at that
statement. Alias chains shall preserve that set, later root bindings shall not
retroactively enter it, and a published root binding shall have identical
single-application lookup behavior without becoming a native export.

Direct root and alias data selection in source-root executable blocks shall
produce a reference to the original binding identity. A lexical binding with
the same source name shall remain distinct, and code generation shall evaluate
the original initializer exactly once. DWARF/GDB shall retain source names and
the observable Scope alias even though LLVM environment keys use the stable
identities. Native tests shall cover alias chains, earlier snapshots versus the
live root, typed Scope aliases, publication, lexical shadow exclusion, exact
output, and freestanding artifact inspection.

This shall add no runtime namespace lookup/table, Scope allocation, foreign
dependency, C/C++ runtime, other-language standard library, public data ABI, or
`topal-native/6` revision. Function-body root-data access, nested qualified Scope
members, general Scope function boundaries, generators, `use`, packages, and
source/compiled libraries remain rejected by increment 6b2a. This realizes
`TOPAL-COMPILER-NAMESPACE-DATA-001` for increment 6b2a under
`TOPAL-NAMESPACE-ROOT-001`, `TOPAL-NAMESPACE-ALIAS-001`,
`TOPAL-NAMESPACE-SNAPSHOT-001`, `TOPAL-NAMESPACE-CLASSIFIER-001`, and
`TOPAL-NAMESPACE-ALIAS-CHAIN-001`.

## TOPAL-COMP-NAMESPACE-BOUNDARY-001 — Specialized Scope parameters

For an ordinary function called with the live source `root` Scope, a retained
root alias, or an already-specialized Scope parameter, the checked model shall
specialize each `Scope` parameter with the concrete namespace identity,
source-position declaration snapshot, function overload sets, and represented
data members. Qualified function calls shall use the existing direct
specialization path. For each non-discarded Scope parameter, the frontend shall
carry every data member with an admitted private representation as an exact
hidden argument, so direct selection and transitive Scope-parameter forwarding
reference the original already-evaluated value without caller-frame lookup. It
shall diagnose a missing selected member and a selected member that lacks an
admitted private representation.

The Linux x86-64 backend shall retain the explicit sealed `i32` Scope value and
append exact typed hidden parameters under private `fastcc`; definitions and
calls shall have identical LLVM types, and LLVM shall own physical register,
stack, and aggregate classification. It shall preserve the explicit Scope
parameter and material hidden arguments in DWARF/GDB at O0. Tests shall cover
the shared interpreter regression, data selection, direct qualified functions,
forwarding, stale alias rejection, single initializer execution, native
artifacts, source frames, and separate resource baselines.

This shall add no runtime namespace lookup/table, indirect dispatch, Scope or
environment allocation, foreign dependency, C/C++ runtime, other-language
standard library, public Scope ABI, or `topal-native/6` revision. Scope results
or escape, function-local live-root arguments, nested/non-root namespaces,
generator members, `use`, and compiled-library environments remain deferred.
Exact root and root-alias package fields are governed by
`TOPAL-COMP-SCOPE-PACKAGED-FIELD-001`; all other Scope package forms remain
deferred. This realizes `TOPAL-COMPILER-NAMESPACE-BOUNDARY-001` and
`TOPAL-NAMESPACE-FUNCTION-BOUNDARY-001` for increment 6b2b1.

## TOPAL-COMP-NAMESPACE-GENERATOR-001 — Static qualified generator application

For a source-root `root` value or retained root alias, the checked compiler
model shall include every generator declaration visible in that namespace
snapshot, preserving namespace identity, source-ordered overloads, declared
directions, checked body graph, and source provenance. A qualified application
shall resolve only in that captured generator set, apply the existing checked
generator rules, evaluate each input exactly once, and create a fresh affine
continuation with the same ownership and close obligations as an unqualified
application. A declaration introduced after an alias binding shall remain
absent from that alias while remaining available through a later live `root`.

The initial Linux x86-64 subset shall qualify every custom-generator graph that
the compiler otherwise admits; the unchanged
`examples/language/namespace-generator.t` regression exercises the existing
single-Character-yield O0 path, and checked tests shall also cover qualified
overload preservation. Namespace selection shall be complete before LLVM
lowering; the backend shall reuse the selected checked graph and inline
suspension traversal while preserving Scope, Generator, yielded values, and
source locations in DWARF/GDB. Native tests shall cover alias and direct-root
selection, snapshot exclusion, fresh linear consumption, exact interpreter
output, freestanding artifacts, and separate resource baselines.

This shall add no runtime namespace lookup/table, continuation object,
callback, indirect dispatch, foreign dependency, C/C++ runtime, other-language
standard library, public generator ABI, or `topal-native/6` revision. A future
compiled-library interface that publishes such a member shall carry stable
namespace and revision identity, visibility, the generator overload signature,
checked directions and body graph, capture/effect/linearity/ownership facts,
close and suspension behavior, semantic representation identity, and a
versioned target adapter; it shall not expose compiler-private tags or symbols
as portable identity. Non-root/external namespaces, Scope parameters or results
whose selected members are generators, generator shapes the compiler does not
otherwise admit, dynamic or escaping qualified generator values, packages, and
source/compiled libraries remain deferred. This realizes
`TOPAL-COMPILER-NAMESPACE-GENERATOR-001` and
`TOPAL-NAMESPACE-GENERATOR-001` for increment 6b2b2a.

## TOPAL-COMP-FUNCTION-ROOT-DATA-001 — Private live-root data capture

For an ordinary root function called directly from the source entry frame, the
checked model shall resolve each exact `root member` data selection against the
live root bindings whose initializers have completed at the call position. It
shall admit supported private machine values even when their declarations
follow the function declaration, isolate the qualified selection from
same-named parameters or lexical captures, and reuse the original once-evaluated
root storage value.

The frontend shall append referenced members in root declaration order as exact
private capture arguments and parameters named `root member`. The Linux x86-64
backend shall emit matching `fastcc` definitions and calls and leave physical
register, stack, and aggregate placement to LLVM. Full O0 DWARF/GDB shall expose
the ordinary function frame, explicit parameters, root capture classifiers, and
values. Tests shall cover the shared interpreter regression, a function declared
before its root data, lexical shadow isolation, initializer order, checked-model
capture order and rejection boundaries, exact IR, freestanding artifacts,
source debugging, the shared corpus, and separate resource baselines.

This shall add no global root storage, namespace/capture table, initializer
replay, lookup, allocation, function pointer, indirect call, foreign dependency,
C/C++ runtime, other-language standard library, public ABI, or
`topal-native/6` revision. Exact scalar forwarding is governed by
`TOPAL-COMP-FUNCTION-ROOT-DATA-FORWARD-001`. Aggregate values beyond
`TOPAL-COMP-AGGREGATE-ENVIRONMENT-001`, callable, Scope, Generator, constraint,
or evidence members; anonymous/escaping functions; other qualified-root forms;
and public/library root environments remain deferred.
Future compiled-library metadata shall preserve canonical
source-session namespace identity, selection/call positions, member stable
identity, visibility/declaration order, classifier/semantic representation,
capture order/lifetime/effects, and a versioned target adapter independently of
private capture names, LLVM types/symbols, and physical placement. This realizes
`TOPAL-COMPILER-FUNCTION-ROOT-DATA-001` and `TOPAL-NAMESPACE-ROOT-001` for
increment 6b2b2b.

## TOPAL-COMP-FUNCTION-ROOT-DATA-FORWARD-001 — Private live-root data forwarding

For a finite acyclic chain of statically named ordinary root functions reached
from the source entry frame, the checked model shall compute transitive exact
`root member` selections before instantiating the outer function. It shall
append every required supported private machine value to each intermediate
function's hidden capture group in root declaration order and forward the same
already-evaluated value at every direct call edge. The live root snapshot shall
be the one visible at the outer entry-frame call position, and same-named
ordinary parameters or locals shall remain isolated in every frame.

The Linux x86-64 backend shall emit matching exact `fastcc` definitions and
calls and leave physical placement to LLVM. Full O0 DWARF/GDB shall expose all
explicit and forwarded parameters accurately in the active function and every
suspended caller frame. Target-aligned debug-only stack shadows may preserve
call-clobbered values without adding semantic storage. Tests shall cover a
three-frame shared regression, interpreter modes, reversible history, checked
capture order and call arguments, unresolved overload-selection rejection,
exact direct IR, artifact-free failure, freestanding ELF/DWARF, every GDB frame,
the shared corpus, and separate resource baselines.

This shall add no global root storage, namespace/capture/environment table,
initializer replay, lookup, allocation, function pointer, indirect call,
foreign dependency, C/C++ runtime, other-language standard library, public ABI,
or `topal-native/6` revision. Overload forwarding beyond
`TOPAL-COMP-OVERLOAD-ENVIRONMENT-001`, recursive forwarding beyond
`TOPAL-COMP-RECURSIVE-SCALAR-ENVIRONMENT-001`, local named Function forwarding
beyond `TOPAL-COMP-LOCAL-FUNCTION-ENVIRONMENT-001`, anonymous functions,
aggregate environments beyond
`TOPAL-COMP-AGGREGATE-ENVIRONMENT-001`, otherwise unsupported root members,
defining-context forwarding beyond
`TOPAL-COMP-CONTEXT-CAPTURE-FORWARD-001`, escape, and
public/library environments remain deferred. Future compiled-library metadata
shall preserve canonical source-session namespace identity, every selection and
call edge, callee identity/overload, member stable identity,
visibility/declaration order, classifier/semantic representation, capture
order/lifetime/effects, and a versioned target adapter independently of private
names, LLVM types/symbols, debug shadows, and physical placement. This realizes
`TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001` and
`TOPAL-NAMESPACE-ROOT-001` for increment 6b2b2c.

## TOPAL-COMP-NAMED-FUNCTION-VALUE-001 — Retained named function values

The checked compiler model shall admit an already-visible ordinary or static
root function in value position, retaining its source identity, visible
declarations, source-ordered overload set, and staticness. Binding and rebinding
that Function value shall preserve the retained candidates. Applying an alias
shall use the original function name for overload, recursion, specialization,
and diagnostics and shall not restart lookup at the alias or caller scope.

The program model shall expose a deterministic module-local name table so
expression tags, canonical `<fn name>` display, and Function DWARF enumerators
agree. LLVM lowering shall nevertheless call the selected private `fastcc`
symbol directly; neither the tag nor a native pointer shall dispatch the call.
Native and GDB tests shall cover exact execution, retained value identity,
typed aliases, binding chains, snapshot overload exclusion, lexical shadowing,
Function display, argument inspection, and source frames.

The implementation shall introduce no indirect call, closure allocation,
Function runtime, foreign dependency, C/C++ runtime, other-language standard
library, public callable ABI, or `topal-native/6` revision. Symbolic callables,
anonymous functions/captures, Function parameters/results, namespace selection
of function-valued data, and published callable interfaces remain rejected.
This realizes `TOPAL-COMPILER-NAMED-FUNCTION-VALUE-001` and the admitted portion
of `TOPAL-FUNCTION-VALUE-001` for compiler increment 3b2-b5j.

## TOPAL-COMP-SYMBOLIC-CALLABLE-VALUE-001 — Direct symbolic Function values

The checked compiler model shall admit `+`, `-`, and `<=>` in value position,
retain the exact symbolic identity through unclassified or `Function`-classified
bindings and binding chains, and reject invalid application arity with the
ordinary no-applicable-overload diagnostic. Bound binary application shall
unpack one two-field positional product before using the existing numeric and
comparison checks; bound `-` shall additionally retain unary negation.

The module-local Function observation table shall include `+`, `-`, and `<=>`
in deterministic order after named functions whenever Function values are
used. Canonical output and DWARF/GDB shall retain those spellings. Code generation
shall lower the selected callable directly to the existing LLVM operation or
Topal-owned numeric primitive and shall not dispatch through the observation
tag. Native tests shall cover exact add, unary/binary subtraction, three-way
comparison, Function classification, binding chains, display, and debugging.

This shall add no function pointer, indirect call, Function runtime, closure
allocation, foreign dependency, C/C++ runtime, other-language standard library,
public callable ABI, or `topal-native/6` revision. Other symbolic callables,
Function parameters/results, anonymous functions/captures, and published
callable interfaces remain rejected. This realizes
`TOPAL-COMPILER-SYMBOLIC-CALLABLE-VALUE-001` and the admitted portion of
`TOPAL-FUNCTION-CALLABLE-VALUE-001` for compiler increment 3b2-b5k.

## TOPAL-COMP-SYMBOLIC-CALLABLE-EXPANDED-001 — Complete symbolic Function values

The checked compiler model shall admit `=`, `!=`, `<`, `>`, `<=`, `>=`, `*`,
`/`, `/%`, `%`, `^`, `..`, `<..`, `..=`, and `<..=` in value position in
addition to the existing `+`, `-`, and `<=>` values. Bindings and private
Function parameter/result specializations shall retain each exact callable.
Binary application shall unpack one two-field positional product and reuse the
existing checked operation so conversions, fallibility, result classifiers,
range endpoint policy, and diagnostics remain identical to direct syntax.

The module-local observation table shall retain the existing first three tags,
append every newly admitted canonical spelling deterministically, and observe
source `!=` as canonical `/=`. LLVM lowering shall emit the already-selected
direct operation or Topal-owned runtime primitive. Native tests shall cover all
new identities, exact interpreter parity, private Function result passage,
stable tags, direct IR, freestanding artifacts, full O0 DWARF/GDB values and
frames, the shared corpus, and separate resource baselines.

This shall add no function pointer, indirect call, Function runtime, closure
allocation, foreign dependency, C/C++ runtime, other-language standard library,
public callable ABI, or `topal-native/6` revision. Dynamic selection, aggregate
containment, escape, publication, and library metadata/adapters remain deferred.
This realizes `TOPAL-COMPILER-SYMBOLIC-CALLABLE-EXPANDED-001`,
`TOPAL-FUNCTION-CALLABLE-VALUE-001`, and `TOPAL-TYPE-CALL-001` for compiler
increment 3b2-b5t.

## TOPAL-COMP-FUNCTION-PARAMETER-001 — Specialized private Function inputs

The checked compiler model shall admit a scalar `Function` parameter for an
admitted named or symbolic Function argument. Caller analysis shall attach the
retained callable facts to that argument, and callee specialization shall bind
those facts to the source parameter. Applying the parameter shall select the
retained declaration vector or symbolic operation without restarting lookup.
Bound arguments shall preserve the same facts, and distinct callable arguments
may instantiate distinct private versions of one source function.

The LLVM parameter shall be the exact private i32 observation tag used by the
caller, while the specialized body shall contain the already-selected direct
call or operation. Since executable computation need not read the tag, full O0
debugging shall retain it in a target-aligned debug-only stack shadow associated
with the source parameter and Function DWARF type. Native and GDB tests shall
cover direct and bound symbolic inputs, retained named inputs, multiple
specializations, exact signature/calls, parameter value, and nested frame.

This shall add no tag dispatch, function pointer, indirect call, closure
allocation, Function runtime, foreign dependency, C/C++ runtime, other-language
standard library, public callable ABI, or `topal-native/6` revision. Function
results, aggregate Function boundaries, anonymous functions/captures, remaining
symbolic callables, and published callable interfaces remain rejected. This
realizes `TOPAL-COMPILER-FUNCTION-PARAMETER-001` and the admitted boundary
portion of `TOPAL-FUNCTION-CALLABLE-VALUE-001`, `TOPAL-FUNCTION-VALUE-001`, and
`TOPAL-TYPE-CALL-001` for compiler increment 3b2-b5l.

## TOPAL-COMP-ANONYMOUS-DIRECT-001 — Private direct anonymous functions

The checked compiler model shall retain an inferred anonymous function with
binding-only parameter patterns as a Function value carrying its source body,
arity, construction identity, and the lexical bindings it would capture. A
non-capturing value shall be applicable after binding or when supplied directly
to an admitted private `Function` parameter. The call site shall infer a unary
parameter from its direct operand or multiple parameters from one exact-arity
positional product, preserving left-to-right evaluation. Body analysis shall
bind those inferred classifiers before inferring its result and shall preserve
left-to-right, no-hidden-precedence symbolic application semantics.

Each application shall specialize one private anonymous `fastcc` function and
emit a direct call. A deterministic module-private Function observation tag
shall provide canonical `<anonymous fn/N>` display and Function DWARF identity
without controlling the call. Full O0 debugging shall expose each anonymous
source frame and inferred parameter names and values. Native tests shall cover
unary and product invocation, exact output, contextual Function-parameter use,
left-associative mixed symbolic application, invalid arity, display, source
frames, private direct IR, and freestanding artifact properties.

This requirement alone does not admit lexical data captures or anonymous
product parameter patterns; unsupported forms shall fail before LLVM lowering.
This increment shall add no function pointer, indirect call, closure allocation,
closure or Function runtime, foreign dependency, C/C++ runtime, other-language
standard library, public callable ABI, or `topal-native/6` revision. Escaping
closures, Function results and aggregate boundaries outside their dedicated
requirements, and published callable interfaces remain rejected. This realizes
`TOPAL-COMPILER-ANONYMOUS-DIRECT-001` and the admitted portion of
`TOPAL-FUNCTION-ANONYMOUS-001`, `TOPAL-SYN-GRAMMAR-001`, and
`TOPAL-TYPE-CALL-001` for compiler increment 3b2-b5m.

## TOPAL-COMP-NESTED-FUNCTION-001 — Private direct nested lexical functions

The checked compiler model shall admit an unpublished ordinary nested function
declared as a direct statement of an ordinary non-static function body. It
shall bind that function from its declaration point, retain its source
declaration, and apply it by direct name within the same invocation scope or by
an exact local alias governed by
`TOPAL-COMP-LOCAL-FUNCTION-ENVIRONMENT-001`.
It shall snapshot every visible immutable lexical data value with an admitted
private representation, excluding names shadowed by explicit nested
parameters, and shall reject any attempted value escape. A call shall reuse
each already-evaluated captured value and shall not access storage in another
native frame.

Each application shall specialize a compiler-private nested `fastcc` function.
Its exact signature shall contain source parameters followed by deterministic
exact typed capture parameters, with the identical prototype at every emitted
definition and call. LLVM shall own physical x86-64 register, stack, and
aggregate placement. Full O0 debugging shall expose the nested source frame,
source parameters, and material captures as named arguments. Native tests shall
cover the existing interpreter regression, exact execution, capture forwarding,
value-escape rejection, private direct IR, freestanding artifacts, source
frames, argument values, the shared corpus, and separate resource baselines.
Exact local alias calls and exact defining-context/live-root parameters are
governed by `TOPAL-COMP-LOCAL-FUNCTION-ENVIRONMENT-001`; those environment
parameters shall not be duplicated as lexical captures.

This shall add no caller-frame reference, environment allocation/runtime,
function pointer, indirect call, foreign dependency, C/C++ runtime,
other-language standard library, public closure ABI, or `topal-native/6`
revision. Declarations inside nested lexical/decision blocks and published,
static, measured, constrained, or effectful nested functions; nested overloads,
recursion, sibling calls, visible/active named-callable collisions, anonymous
captures, Scope/Function/Constraint/refined captures, context/root environments
beyond `TOPAL-COMP-LOCAL-FUNCTION-ENVIRONMENT-001`, escaping closures, and
public/library closure metadata remain rejected. This realizes
`TOPAL-COMPILER-NESTED-FUNCTION-001` and `TOPAL-FUNCTION-NESTED-001` for compiler
increment 3b2-b5p.

## TOPAL-COMP-FUNCTION-RESULT-001 — Specialized private Function results

The checked compiler model shall admit an exact `Function` result from an
ordinary or static root function when the result has retained named-root or
symbolic callable facts already supported by the compiler. A specialized
`Function` parameter may pass those same facts through the result. Calls and
subsequent binding chains shall retain the callable facts independently of the
machine tag, and later application shall lower from those facts without
runtime dispatch or renewed name lookup.

Each specialization shall return the existing deterministic i32 Function
observation tag from its exact private `fastcc` signature. The caller shall use
a direct i32-returning call, and named or symbolic application shall remain a
direct private call or direct operation. Full O0 debugging shall expose the
Function parameter, returned Function locals, and source frames using
target-aligned debug-only stack shadows where otherwise-dead tags would lose a
stable GDB location. Native tests shall cover named and symbolic pass-through,
binding chains, exact output, rejection boundaries, private direct IR,
freestanding artifacts, DWARF validation, GDB values/frames, the shared corpus,
and separate resource baselines.

Capturing anonymous results are governed by
`TOPAL-COMP-FUNCTION-CAPTURE-RESULT-001`; exact aggregate containment is
governed by `TOPAL-COMP-FUNCTION-AGGREGATE-001` and
`TOPAL-COMP-FUNCTION-AGGREGATE-CAPTURE-001`. Nested, dynamically computed, and
published Function results outside those rules remain rejected. This shall add
no function pointer, indirect call, closure
allocation/runtime, foreign dependency, C/C++ runtime, other-language standard
library, public callable ABI, or `topal-native/6` revision. This realizes
`TOPAL-COMPILER-FUNCTION-RESULT-001` and the admitted
result-boundary portions of `TOPAL-FUNCTION-CALLABLE-VALUE-001`,
`TOPAL-FUNCTION-VALUE-001`, and `TOPAL-TYPE-CALL-001` for compiler increment
3b2-b5q.

## TOPAL-COMP-ANONYMOUS-CAPTURE-001 — Private anonymous captures and results

The checked compiler model shall admit a bound inferred anonymous function that
captures immutable lexical data with complete admitted private representations
when it is applied directly in the same defining invocation. It shall retain
the construction-time storage identities, append their already-evaluated values
in deterministic lexical-name order after the explicit operands, and bind the
anonymous body only to exact hidden capture parameters. A missing defining
storage identity, caller shadow, callable or namespace capture, or unsupported
representation shall fail before LLVM lowering.

An ordinary or static root function may return a non-capturing inferred
anonymous Function. Result callable facts shall retain its body, source arity,
construction identity, static context, and empty capture set separately from
the returned observation tag. A later binding and application shall specialize
that body and produce one direct private call. Capturing anonymous Functions at
private Function parameters are governed by
`TOPAL-COMP-FUNCTION-CAPTURE-PARAMETER-001`; admitted private capturing results
are governed by `TOPAL-COMP-FUNCTION-CAPTURE-RESULT-001`.

LLVM definitions and calls shall use matching private `fastcc` signatures with
source parameters followed by exact capture parameters, leaving physical
x86-64 placement to LLVM. A non-capturing result shall retain the existing i32
Function return. Full O0 DWARF/GDB shall expose the anonymous frame, explicit
parameters, source-named captures, and returned Function local. Native tests
shall cover exact output, checked capture/result models, unsupported escapes,
exact IR, direct calls, artifact independence, DWARF validation, debugger
values/frames, the shared corpus, and separate resource baselines.

This shall add no environment object, function pointer, indirect call, closure
or Function runtime, foreign dependency, C/C++ runtime, other-language standard
library, public callable ABI, or `topal-native/6` revision. Capturing results or
Function-boundary arguments outside
`TOPAL-COMP-FUNCTION-CAPTURE-RESULT-001` and
`TOPAL-COMP-FUNCTION-CAPTURE-PARAMETER-001`, capture-bearing aggregate
containment outside `TOPAL-COMP-FUNCTION-AGGREGATE-CAPTURE-001`, dynamic escape,
publication, and library metadata/adapters remain deferred.
This realizes `TOPAL-COMPILER-ANONYMOUS-CAPTURE-001`,
`TOPAL-FUNCTION-ANONYMOUS-001`, `TOPAL-FUNCTION-VALUE-001`, and
`TOPAL-TYPE-CALL-001` for compiler increment 3b2-b5r.

## TOPAL-COMP-FUNCTION-CAPTURE-PARAMETER-001 — Private captured Function parameters

The checked compiler model shall admit a capturing anonymous Function or a
non-escaping nested lexical Function at an ordinary private `Function`
parameter. The exact callable facts shall remain separate from the i32
observation tag. Every immutable capture shall already have an admitted private
function-boundary representation and shall contain neither Generator nor
Function. The caller shall forward the already-evaluated capture values after
the source parameters in deterministic retained order. A specialized callee
may forward the same facts and values through another ordinary Function
parameter without re-evaluation while the defining lifetime remains active.

The frontend shall remap each capture storage identity to an exact hidden
parameter at every boundary, reject missing or shadowed storage, and specialize
the retained anonymous or nested body at its eventual application. A nested
Function value may receive one deterministic module-private observation tag,
but lowering shall emit only direct calls. Hidden forwarding parameters shall
remain absent from source DWARF, while the Function parameter, actual anonymous
or nested frame, explicit operands, and material source-named captures remain
inspectable.

Tests shall cover multiple scalar captures, an admitted Tuple capture, a root
anonymous capture, function-local anonymous and nested construction, transitive
forwarding, exact output in every interpreter mode, reversible debugging,
checked capture remapping, direct private IR, captured-result composition,
freestanding artifact properties, full O0 GDB values/frames, the shared corpus,
and separate resource baselines. This shall add no environment object or
allocation, function pointer, indirect call, callback, closure or Function
runtime, foreign dependency, C/C++ runtime, other-language standard library,
public callable ABI, or `topal-native/6` revision. Capturing Function results
outside `TOPAL-COMP-FUNCTION-CAPTURE-RESULT-001`, capture-bearing aggregate
containment outside `TOPAL-COMP-FUNCTION-AGGREGATE-CAPTURE-001`, dynamic
selection or escape, recursive or overloaded nested callable values,
unsupported captured state, publication, and library metadata/adapters remain
deferred. Future library metadata shall describe
callable and capture identities, ordered classifiers, lifetime/effects,
representation identities, and target adapters rather than the private hidden-
parameter layout. This realizes
`TOPAL-COMPILER-FUNCTION-CAPTURE-PARAMETER-001`,
`TOPAL-FUNCTION-ANONYMOUS-001`, `TOPAL-FUNCTION-NESTED-001`,
`TOPAL-FUNCTION-VALUE-001`, and `TOPAL-TYPE-CALL-001` for compiler increment
3b2-b5w.

## TOPAL-COMP-FUNCTION-CAPTURE-RESULT-001 — Private captured Function results

The checked compiler model shall admit a capturing anonymous Function as the
exact result of an ordinary or static private specialization when its body,
construction identity, and all immutable capture facts remain known. Every
capture shall have an already-admitted complete private function-boundary
representation and contain neither Generator nor Function. A specialized
Function parameter carrying those facts may be returned, and an admitted
captured result may be forwarded through another admitted private Function
result or parameter.

Each generated factory shall return one exact aggregate containing the i32
Function observation tag followed by the already-evaluated capture values in
retained order. Its caller shall issue one direct `fastcc` call, decompose the
aggregate into deterministic compiler-only SSA storage, and retain the callable
facts separately. Eventual application shall specialize the anonymous body and
pass those returned values after its explicit operands. It shall neither replay
capture initializers nor dispatch on the observation tag. LLVM shall own the
physical x86-64 aggregate return convention.

Source DWARF shall continue to describe the factory result and binding as
`Function`; private returned transport fields shall have no source variables.
The eventual anonymous frame shall expose explicit operands and material
captures under their source names. Tests shall cover scalar and Tuple captures,
root and function-local factories, Function-parameter pass-through, transitive
result forwarding, exact output in every interpreter mode, reversible
debugging, checked capture remapping, exact direct IR, unsupported-result
edges, freestanding artifacts, full O0 GDB values/frames, the shared corpus,
and separate resource baselines.

This shall add no heap or environment object, environment pointer, allocation,
function pointer, indirect call, callback, closure or Function runtime, foreign
dependency, C/C++ runtime, other-language standard library, public callable
ABI, or `topal-native/6` revision. Immediate exact result application is
governed by `TOPAL-COMP-FUNCTION-RESULT-CHAIN-001`. Exact nested Function escape
is governed by `TOPAL-COMP-NESTED-FUNCTION-ESCAPE-001`. Function-valued or
otherwise unsupported captures, dynamic selection,
capture-bearing aggregate containment outside
`TOPAL-COMP-FUNCTION-AGGREGATE-CAPTURE-001`, publication, and library
metadata/adapters remain deferred. Future library metadata shall describe
callable identity, ordered capture identities/classifiers, lifetime/effects,
representation identities, and target adapters rather than this private
aggregate transport.
This realizes `TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001`,
`TOPAL-FUNCTION-ANONYMOUS-001`, `TOPAL-FUNCTION-VALUE-001`, and
`TOPAL-TYPE-CALL-001` for compiler increment 3b2-b5x.

## TOPAL-COMP-FUNCTION-RESULT-CHAIN-001 — Exact Function-result application chains

The checked compiler model shall fold left-associative application when an
admitted application returns an exact Function and a following operand applies
that result. It shall support flat prefix and explicitly parenthesized chains
for retained named, symbolic, non-capturing anonymous, and admitted capturing
anonymous results. Each intermediate shall be evaluated once in source order,
with callable facts retained separately from the observation tag in
compiler-only storage. Captured aggregate fields extracted from a result shall
remain available to the immediate anonymous specialization.

LLVM shall receive only the existing exact direct `fastcc` definitions and
calls. The observation tag shall not dispatch application, and neither the
factory nor capture initializers may be replayed. A non-Function intermediate
or incompatible retained operand shall be rejected before lowering. Source
DWARF shall expose the ordinary factory, returned named function, and anonymous
capture frames but no compiler-only chain binding.

Tests shall cover flat and parenthesized chains, scalar and Tuple captures,
closed anonymous results, named and symbolic pass-through, scalar and product
operands, exact output in every interpreter mode, reversible debugging,
checked-model structure, once-only direct IR, rejection after a non-Function
intermediate, freestanding artifacts, full O0 GDB values/frames, the shared
corpus, and separate resource baselines. This shall add no allocation,
environment object, function pointer, indirect call, callback, closure or
Function runtime, foreign dependency, C/C++ runtime, other-language standard
library, public callable ABI, or `topal-native/6` revision. Aggregate-contained
Function values outside `TOPAL-COMP-FUNCTION-AGGREGATE-001`, dynamically
selected Function values, nested escape outside
`TOPAL-COMP-NESTED-FUNCTION-ESCAPE-001`, publication, and library
metadata/adapters remain deferred. This realizes
`TOPAL-COMPILER-FUNCTION-RESULT-CHAIN-001`,
`TOPAL-FUNCTION-CALLABLE-VALUE-001`, `TOPAL-FUNCTION-VALUE-001`, and
`TOPAL-TYPE-CALL-001` for compiler increment 3b2-b5y.

## TOPAL-COMP-FUNCTION-AGGREGATE-001 — Exact private Function aggregates

The checked compiler model shall retain recursive structural facts for Tuple
and Record values containing Function fields. Each Function leaf shall retain
one exact named, symbolic, or anonymous callable identity separately from its
private observation tag. Binding, direct Record selection, and anonymous
product destructuring shall recover those facts without replaying aggregate or
field construction. Local selection may apply a capturing anonymous Function
while all captures remain in their defining lifetime.

The capture-free base case permits an ordinary private function parameter or
result to carry a recursively nested Tuple or Record when every Function leaf
is exact at the call site. Specialization shall propagate those recursive facts
into the callee or back to the caller. Capture-bearing aggregate boundaries are
governed by `TOPAL-COMP-FUNCTION-AGGREGATE-CAPTURE-001`. Missing, opaque, or
branch-selected facts and Function containment outside Tuple, Record, the exact
Optional path of `TOPAL-COMP-OPTIONAL-FUNCTION-001`, or the exact nominal Sum
path of `TOPAL-COMP-SUM-FUNCTION-001`, or the exact Result-success path of
`TOPAL-COMP-RESULT-FUNCTION-001`, or the exact finite List-entry paths of
`TOPAL-COMP-LIST-FUNCTION-001`, or the exact fixed-size Array-entry paths of
`TOPAL-COMP-ARRAY-FUNCTION-001`, or the exact String-keyed Map-value paths of
`TOPAL-COMP-MAP-FUNCTION-001` shall fail before LLVM lowering.

The backend shall represent each Function leaf with its existing private i32
observation tag inside the recursively exact aggregate. Definitions and calls
shall use matching direct `fastcc` prototypes, while LLVM owns physical x86-64
aggregate argument and result coercion. Application shall use retained checked
facts to emit the selected direct operation or function call; tags shall never
dispatch. DWARF/GDB shall expose recursively accurate Tuple/Record source
values and their Function identities.

Tests shall cover local captured containment; named, symbolic, and
non-capturing anonymous values; recursive Tuple/Record bindings, parameters,
results, selections, and destructuring; checked structure; exact direct IR;
every interpreter mode and reversible history; the shared corpus and separate
resource baselines; freestanding ELF and DWARF validation; and full O0 GDB
values/frames.

This shall add no closure object, environment pointer, allocation, function
pointer, indirect call, callback, dispatch table, foreign dependency, C/C++
runtime, other-language standard library, public aggregate/callable ABI, or
`topal-native/6` revision. Dynamic aggregate boundaries, capture-bearing
boundaries outside `TOPAL-COMP-FUNCTION-AGGREGATE-CAPTURE-001`, other aggregate
constructors, escape, publication, and library adapters remain deferred. Future
compiled-library metadata shall encode every Function field's canonical
aggregate path, callable identity, ordered captures and classifiers,
construction lifetime, effects, representation identity, and target adapter
independently of private LLVM types and observation tags. This realizes
`TOPAL-COMPILER-FUNCTION-AGGREGATE-001`,
`TOPAL-ABSTRACTION-FUNCTION-BOUNDARY-001`, `TOPAL-FUNCTION-VALUE-001`,
`TOPAL-FUNCTION-ANONYMOUS-001`, `TOPAL-FUNCTION-CALLABLE-VALUE-001`,
`TOPAL-TYPE-PRODUCT-001`, and `TOPAL-TYPE-CALL-001` for compiler increment
3b2-b5aa.

## TOPAL-COMP-FUNCTION-AGGREGATE-CAPTURE-001 — Captured private Function aggregates

The checked compiler model shall admit exact recursively nested Tuple and
Record parameters whose Function leaves are capturing anonymous Functions or
nested Functions, and results whose Function leaves are capturing anonymous
Functions or exact nested Functions under
`TOPAL-COMP-NESTED-FUNCTION-ESCAPE-001`. Every leaf shall retain its exact
callable identity, ordered capture facts, and canonical zero-based-Tuple-index,
Record-label, Optional-payload, nominal-Sum-alternative, Result-success,
zero-based List/Array-entry, or semantic exact String-keyed Map-value path.
Paths and their captures shall be traversed depth-first from left to right; Map
paths shall use first-key occurrence order after collision resolution. Optional,
Sum, Result, List, Array, and Map edges shall satisfy their respective
exact-container requirements.

For each specialized parameter, the frontend shall append every path-ordered
capture after the source-visible aggregate argument and bind those operands to
the selected callable's capture storage. For each result, it shall retain the
source aggregate and append the same path-ordered capture values to one
compiler-private result aggregate. Calls, bindings, transitive forwarding,
Record selection, and recursive anonymous product destructuring shall recover
the captures without replaying the aggregate, fields, or capture initializers.
Function-containing capture state, Generator state, opaque or dynamically
selected callable facts, and values outside the represented private lifetime
shall fail before LLVM lowering.

The backend shall emit matching direct private `fastcc` definitions and calls,
leaving x86-64 physical argument/result placement to LLVM and the qualified
target data layout. Source aggregates shall retain recursively accurate DWARF
types and Function members. Hidden transport shall not appear as source
aggregate fields or compiler-named variables; invoked anonymous and nested
frames shall expose captures under source names.

Tests shall cover multiple captured Record leaves, a captured Tuple leaf,
parameter-to-result forwarding, recursive destructuring, a nested Function
parameter, exact nested escape under `TOPAL-COMP-NESTED-FUNCTION-ESCAPE-001`,
rejected Function-containing capture state, checked path ordering, exact direct
IR, all interpreter modes and reversible history, the shared corpus and separate
resource baselines, freestanding ELF/DWARF validation, and full O0 GDB aggregate
values and capture frames.

This shall add no closure object, environment pointer, allocation, function
pointer, indirect call, callback, dispatch table, foreign dependency, C/C++
runtime, other-language standard library, public aggregate/callable ABI, or
`topal-native/6` revision. Dynamic Function selection, other aggregate
constructors, escaping nested Functions outside
`TOPAL-COMP-NESTED-FUNCTION-ESCAPE-001`, persistent storage, publication, and
public library adapters remain deferred. Future compiled-library metadata shall
encode the canonical aggregate path; callable and capture identities; capture
classifiers and order; construction lifetime; effects; representation identity;
and target adapter independently of private LLVM aggregates, observation tags,
and the current hidden-operand layout. This realizes
`TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`,
`TOPAL-COMPILER-FUNCTION-AGGREGATE-001`,
`TOPAL-ABSTRACTION-FUNCTION-BOUNDARY-001`, `TOPAL-FUNCTION-VALUE-001`,
`TOPAL-FUNCTION-ANONYMOUS-001`, `TOPAL-FUNCTION-NESTED-001`,
`TOPAL-TYPE-PRODUCT-001`, and `TOPAL-TYPE-CALL-001` for compiler increment
3b2-b5ab.

## TOPAL-COMP-NESTED-FUNCTION-ESCAPE-001 — Exact private nested Function escape

The checked compiler model shall admit an exact nonrecursive, nonoverloaded
nested ordinary Function as a scalar Function result, a Function leaf of a
recursively nested Tuple or labeled Record result, or an exact present Optional
payload under `TOPAL-COMP-OPTIONAL-FUNCTION-001`, an exact selected Sum payload
under `TOPAL-COMP-SUM-FUNCTION-001`, or an exact arithmetic Result success under
`TOPAL-COMP-RESULT-FUNCTION-001`, or exact finite List entries under
`TOPAL-COMP-LIST-FUNCTION-001`, or exact fixed-size Array entries under
`TOPAL-COMP-ARRAY-FUNCTION-001`, or exact String-keyed Map values under
`TOPAL-COMP-MAP-FUNCTION-001` within one compilation unit.
It shall retain the nested declaration identity separately from its observation
tag and retain each already-evaluated immutable lexical, defining-context, and
live-root capture with its exact classifier, storage identity, and source name.
Function- or Generator-containing capture state, missing lifetime storage,
opaque facts, and dynamic selection shall remain `E-COMPILER-UNSUPPORTED`.
Separate factory invocations shall retain independent capture snapshots.

Scalar results shall reuse the exact tag-plus-capture return representation of
`TOPAL-COMP-FUNCTION-CAPTURE-RESULT-001`. Tuple, Record, Optional, Sum, and
Result results shall reuse
the canonical Function-leaf path ordering and extended result aggregate of
`TOPAL-COMP-FUNCTION-AGGREGATE-CAPTURE-001`. The caller shall extract every
field once into compiler-only SSA storage. Binding, private Function
parameter/result forwarding, aggregate forwarding/selection/destructuring, and
immediate result application shall remap those values without replaying the
factory, aggregate, or capture initializers.

The backend shall emit matching direct private `fastcc` prototypes and calls,
with the explicit source operands followed by the returned captures. It shall
never dispatch on the Function tag or consult the ended factory frame. LLVM
shall own AMD64 aggregate classification and physical placement from the
qualified target data layout, and correctness shall require no optimization.
Source DWARF shall expose the factory result and bindings as `Function`, retain
ordinary Tuple/Record layouts, omit hidden transport fields, and expose nested
parameters and captures under source names in active and suspended frames.

Tests shall cover distinct factory invocations; scalar, Tuple, context, and
root captures; scalar and Record results; transitive forwarding; immediate
application; exact interpreter output and reversible history; checked capture
paths; direct IR; unsupported Function-valued capture rejection without
artifacts; freestanding ELF/DWARF; full O0 GDB values/frames; the shared corpus;
and separate interpreter/compiler resource baselines.

This shall add no heap, closure/environment object, environment pointer,
allocation, function pointer, indirect call, callback, runtime dispatcher,
global context/root state, caller-frame lookup, foreign dependency, C/C++
runtime, other-language standard library, public ABI, or `topal-native/6`
revision. Recursive, overloaded, sibling-dependent, opaque, dynamically
selected, persistently stored, or published nested Functions remain deferred.
Future library metadata shall retain canonical source-session, lexical-scope,
nested-declaration, callable, aggregate-path, capture
identity/order/classifier/representation/lifetime, effect, and versioned
target-adapter facts independently of observation tags, compiler-private names,
LLVM types/symbols, debug shadows, and physical placement. This realizes
`TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001` for increments 3b2-b5ar, 6b2b2i,
and 6c2g.

## TOPAL-COMP-OPTIONAL-FUNCTION-001 — Exact private Optional Function environments

The checked compiler model shall admit an exact `Optional Function` as a local
value, ordinary private parameter or result, package field, or recursively
contained Tuple/Record field. It shall represent exact absence explicitly and,
for `Some`, retain one exact named, symbolic, anonymous, or admitted nested
callable identity independently of both runtime tags. Opaque or branch-selected
present identity shall remain `E-COMPILER-UNSUPPORTED` before artifact
publication.

The backend shall use the existing Topal Optional header and allocator. A
present payload shall box the existing private i32 Function observation value;
an absent value shall have no payload. An exhaustive decision shall attach the
retained present callable facts to its `Some` binding and lower eventual
application as one direct specialized `fastcc` call. Display, DWARF, and the GDB
printer shall retain the source `Optional Function` classifier and exact
Function observation without exposing compiler facts or hidden captures.

For a present capturing callable, the checked model shall add an Optional
payload edge to its canonical Function path. Private parameters shall pass the
source Optional pointer followed by ordered captures. Private results shall
return that pointer followed by the same captures; the caller shall extract and
remap them exactly once. This path shall compose with exact nested Function
escape and recursively enclosing Tuple/Record values. Function- or
Generator-containing capture state shall remain unsupported.

Repeated anonymous-pattern identity shall compare Optional presence and the
boxed Function observation before any captures. If both present payloads retain
the same callable identity, the compiler shall append and compare corresponding
captures with admitted exact equality. A different callable or presence shall
mismatch without requiring an unselected capture schema. No user Equality or
allocation identity shall participate.

Tests shall cover named, symbolic, anonymous, and nested present payloads;
absence; independent captured factories; direct `Some` application; private
parameter/result, package, Tuple, and Record passage; repeated match/mismatch;
display; all interpreter modes and reversible history; checked Optional paths;
direct IR; artifact-free opaque/capture rejection; freestanding ELF/DWARF;
full O0 GDB Optional/callable frames; the shared corpus; and separate
interpreter/compiler resource baselines.

This shall add no closure/environment object, environment pointer, function
pointer, indirect call, callback, dispatch table, caller-frame lookup, foreign
dependency, C/C++ runtime, other-language standard library, public ABI, or
`topal-native/6` revision. Allocation shall remain the existing Topal-owned
process-lifetime Optional representation. Opaque or dynamically selected
Optional callables, persistent storage, publication, and Function containment
in containers other than exact nominal Sums governed by
`TOPAL-COMP-SUM-FUNCTION-001` remain deferred. Future library metadata shall retain
canonical source-session/scope, callable/declaration, Optional payload and
enclosing aggregate paths, presence/selection proof, ordered capture
identity/classifier/representation/lifetime, effect, container representation,
and versioned target-adapter facts independently of private observation tags,
header layout, compiler names, LLVM types/symbols, debug shadows, and physical
placement. This realizes `TOPAL-COMPILER-OPTIONAL-FUNCTION-001` for increments
3b2-b5as, 6b2b2j, and 6c2h.

## TOPAL-COMP-SUM-FUNCTION-001 — Exact private nominal Sum Function environments

The checked compiler model shall admit an exact labeled Union or positional
Variant with a Function-bearing payload as a local value, ordinary private
parameter or result, package field, or recursively contained Tuple/Record
field. It shall retain the exact selected alternative and complete active
payload facts independently of the runtime tag. Every active Function leaf
shall retain one exact named, symbolic, anonymous, or admitted nested callable
identity. Opaque or branch-selected alternatives or callables shall remain
`E-COMPILER-UNSUPPORTED` before artifact publication.

The backend shall preserve the existing exact private tag-plus-payload Sum
aggregate. A complete decision shall attach retained active payload facts to
its binding and lower eventual Function application as one direct specialized
`fastcc` call. It shall add the semantic alternative name to each active
Function leaf's canonical capture path. Private parameters shall pass the
source aggregate followed by ordered captures; private results shall return the
aggregate followed by those captures for one-time caller extraction and
remapping. The path shall compose with enclosing Tuple/Record values and exact
nested Function escape.

Repeated anonymous-pattern identity shall compare nominal tags and only the
active payload, including the Function observation before captures. When both
operands retain the same selected alternative and callable identity, the
compiler shall compare corresponding captures using admitted exact equality.
A different tag or callable shall mismatch without observing inactive slots or
granting general source Equality to the Function-bearing Sum.

Tests shall cover labeled and positional Sums; named, symbolic, anonymous, and
nested Function payloads; payload-free and non-Function alternatives; distinct
factory snapshots; decision application; private parameter/result, package,
Tuple, and Record passage; repeated match/mismatch; display; all interpreter
modes and reversible history; exact checked paths and direct IR; artifact-free
opaque/capture rejection; freestanding ELF/DWARF; full O0 GDB Sum/callable
frames; the shared corpus; and separate interpreter/compiler resource
baselines.

This shall add no closure/environment object, environment pointer, function
pointer, indirect call, callback, dispatch table, caller-frame lookup,
allocation, foreign dependency, C/C++ runtime, other-language standard library,
public ABI, or `topal-native/6` revision. Function- or Generator-containing
capture state, recursive Sums, opaque or dynamically selected Sum callables,
persistent storage, publication, and Function containment in other containers
remain deferred. Future library metadata shall retain canonical
source-session/scope and nominal Sum identity, labeled/positional form, ordered
alternative/payload schema, active-alternative proof, callable/declaration and
aggregate paths, ordered capture identity/classifier/representation/lifetime,
effects, Sum representation, and versioned target-adapter facts independently
of private tags, inactive layout, compiler names, LLVM types/symbols, debug
shadows, and physical placement. This realizes
`TOPAL-COMPILER-SUM-FUNCTION-001` for increments 3b2-b5at, 6b2b2k, and 6c2i.

## TOPAL-COMP-RESULT-FUNCTION-001 — Exact private Result Function environments

The checked compiler model shall admit an exact
`Result (Function, lang arithmetic ArithmeticErrorCode)` as a local value,
ordinary private parameter or result, package field, or recursively contained
Tuple/Record field. It shall retain one exact named, symbolic, anonymous, or
admitted nested callable identity and complete capture facts conditionally for
the success payload while allowing the runtime success/Error tag to remain
dynamic. Opaque or branch-selected success callables shall remain
`E-COMPILER-UNSUPPORTED` before artifact publication.

The backend shall preserve the existing Topal-owned Result pointer. Success
shall box the private i32 Function observation value; failure and contextual
projection shall preserve the complete original Error. A complete decision
shall attach retained callable facts only to its `Ok` binding and lower
eventual application as one direct specialized `fastcc` call. It shall add a
Result-success edge to the callable capture path. Private parameters shall pass
the source pointer followed by ordered captures; private results shall return
the pointer followed by those captures for one-time caller extraction and
remapping.

Every Error exit from a capture-bearing Result Function shall return the same
private aggregate shape with representation-valid zero carriers in hidden
capture fields. It shall not evaluate skipped callable or capture initializers,
and neither caller nor callee shall expose or consume those carriers on the
Error path. Source Result values, structured Error provenance, successful
Function observations, and eventual source-named captures shall remain fully
visible to DWARF/GDB without exposing hidden fields.

Tests shall cover named, symbolic, anonymous, and nested success payloads; a
dynamically propagated original Error; distinct factory snapshots; `Ok`
application and `Error` handling; private parameter/result, package, Tuple, and
Record passage; display; all interpreter modes and reversible history; checked
Result-success paths and direct IR including aggregate Error exits;
artifact-free opaque/capture rejection; freestanding ELF/DWARF; full O0 GDB
Result/callable frames; the shared corpus; and separate interpreter/compiler
resource baselines.

This shall add no closure/environment object, environment pointer, function
pointer, indirect call, callback, dispatch table, caller-frame lookup, foreign
dependency, C/C++ runtime, other-language standard library, public ABI, or
`topal-native/6` revision. Allocation shall remain limited to the existing
process-lifetime Result object and successful i32 payload box. Function- or
Generator-containing capture state, opaque or dynamically selected success
callables, general Result Function equality/repeated identity, recursive Result
Function payloads, persistent storage, publication, and Function containment
in other containers remain deferred. Future library metadata shall retain the
canonical source-session/scope, Result success classifier and Error-code
vocabulary, conditional-success proof, Error propagation semantics,
callable/declaration and aggregate paths, ordered capture
identity/classifier/representation/lifetime, effects, Result representation,
and versioned target-adapter facts independently of private headers,
observation tags, failure-path zero carriers, compiler names, LLVM
types/symbols, debug shadows, and physical placement. This realizes
`TOPAL-COMPILER-RESULT-FUNCTION-001` for increments 3b2-b5au, 6b2b2l, and
6c2j.

## TOPAL-COMP-LIST-FUNCTION-001 — Exact private finite List Function environments

The checked compiler model shall admit an exact finite `List Function`
constructed from `Entry` and `Empty` as a local value, ordinary private
parameter or result, package field, or recursively contained Tuple/Record
field. It shall retain exact length and one source-ordered named, symbolic,
anonymous, or admitted nested callable identity per entry. Opaque,
branch-selected, transformed, or otherwise inexact entry sequences or
identities shall remain `E-COMPILER-UNSUPPORTED` before artifact publication.

The backend shall preserve Topal-owned singly linked List nodes. A Function
node shall store the private i32 observation followed by the aligned next
pointer; `Empty` shall remain null, and capture snapshots shall not be embedded
in source nodes. A complete List decision shall attach the first entry facts to
the first binding and the remaining ordered facts to the rest binding, then
lower eventual application as one direct specialized `fastcc` call. Display,
DWARF, and the GDB printer shall retain exact source List and Function
observations without exposing compiler facts.

Every Function entry shall add its zero-based source index to its canonical
capture path. Private parameters shall pass the source List pointer followed by
entry- and capture-ordered values. Private results shall return the pointer
followed by those values for one-time caller extraction and remapping. This path
shall compose with enclosing Tuple/Record values and exact nested Function
escape. Distinct entries and factory calls shall retain independent snapshots.
Function- or Generator-containing capture state shall remain unsupported.

Tests shall cover empty, named, symbolic, anonymous, and nested entries;
multiple positions and independent factory snapshots; complete decision
application; private parameter/result, package, Tuple, and Record passage;
display; all interpreter modes and reversible history; checked List-entry paths
and direct IR; artifact-free opaque/capture/repeated-identity rejection;
freestanding ELF/DWARF; full O0 GDB List/callable frames; the shared corpus;
and separate interpreter/compiler resource baselines.

This shall add no closure/environment object, environment pointer, function
pointer, indirect call, callback, dispatch table, caller-frame lookup, foreign
dependency, C/C++ runtime, other-language standard library, public ABI, or
`topal-native/6` revision. Allocation shall remain the existing Topal-owned
process-lifetime List nodes. General List Function equality/repeated identity,
operations that lose exact entry facts, opaque or dynamically selected entries,
persistent storage, publication, recursive element classifiers, and Function
containment in other collection representations remain deferred. Future
library metadata shall retain canonical source-session/scope, exact length and
entry order, callable/declaration and aggregate paths, ordered capture
identity/classifier/representation/lifetime, effects, List representation and
ownership, and versioned target-adapter facts independently of node offsets,
private observation tags, compiler names, LLVM types/symbols, debug shadows,
and physical placement. This realizes `TOPAL-COMPILER-LIST-FUNCTION-001` for
increments 3b2-b5av, 6b2b2m, and 6c2k.

## TOPAL-COMP-ARRAY-FUNCTION-001 — Exact private fixed-size Array Function environments

The checked compiler model shall admit an exact finite `List Function`
collected as `Array N Function`, where `N` is the exact retained count, as a
local value, ordinary private parameter or result, package field, or recursively
contained Tuple/Record field. It shall retain the exact extent and one
source-ordered named, symbolic, anonymous, or admitted nested callable identity
per entry. Opaque, branch-selected, transformed, or otherwise inexact entry
sequences or identities shall remain `E-COMPILER-UNSUPPORTED` before artifact
publication.

The backend shall preserve the existing Topal-owned 16-byte Array header: an
i64 entry count followed by a pointer to the immutable source List nodes.
Collection shall not copy or enlarge those nodes, and capture snapshots shall
not be embedded in the header or entries. Exact checked access shall produce
the ordinary `Optional Function`: an in-bounds index shall attach the indexed
callable facts to `Some`, while an out-of-bounds index shall produce `None`.
Eventual application shall lower as one direct specialized `fastcc` call.
Display, DWARF, and the GDB printer shall retain exact source Array and Function
observations without exposing compiler facts.

Every Function entry shall add its zero-based Array index to its canonical
capture path. Private parameters shall pass the source Array pointer followed
by entry- and capture-ordered values. Private results shall return the pointer
followed by those values for one-time caller extraction and remapping. This path
shall compose with enclosing Tuple/Record values and exact nested Function
escape. Distinct entries and factory calls shall retain independent snapshots.
Function- or Generator-containing capture state shall remain unsupported.

Tests shall cover zero and nonzero extents; named, symbolic, anonymous, and
nested entries; multiple positions and independent factory snapshots;
in-bounds and out-of-bounds access with complete Optional decisions; count,
emptiness, private parameter/result, package, Tuple, and Record passage;
display; all interpreter modes and reversible history; checked Array-entry
paths and direct IR; artifact-free opaque/capture/repeated-identity rejection;
freestanding ELF/DWARF; full O0 GDB Array/callable frames; the shared corpus;
and separate interpreter/compiler resource baselines.

This shall add no closure/environment object, environment pointer, function
pointer, indirect call, callback, dispatch table, caller-frame lookup, foreign
dependency, C/C++ runtime, other-language standard library, public ABI, or
`topal-native/6` revision. Allocation shall remain the existing process-lifetime
List nodes, one Array header, and the ordinary in-bounds Optional Function box.
General Array Function equality/repeated identity, dynamic indexing, operations
that lose exact entry facts, opaque or dynamically selected entries, persistent
storage, publication, recursive element classifiers, and Function containment
in other collection representations remain deferred. Future library metadata
shall retain canonical source-session/scope, exact extent and entry order,
source collection relationship, callable/declaration and aggregate paths,
ordered capture identity/classifier/representation/lifetime, effects,
Array/List representation and ownership, and versioned target-adapter facts
independently of headers, node offsets, private observation tags, compiler
names, LLVM types/symbols, debug shadows, and physical placement. This realizes
`TOPAL-COMPILER-ARRAY-FUNCTION-001` for increments 3b2-b5aw, 6b2b2n, and 6c2l.

## TOPAL-COMP-MAP-FUNCTION-001 — Exact private String-keyed Map Function environments

The checked compiler model shall admit an exact nonempty
`List (String, Function)` collected as `Map (String, Function)` under
`reject`, `keep-first`, or `keep-last` as a local value, ordinary private
parameter or result, package field, or recursively contained Tuple/Record
field. It shall require every source key to be exact, resolve the collision
policy, and retain one named, symbolic, anonymous, or admitted nested callable
identity per surviving key in first-key occurrence order. Empty, opaque,
computed-key, branch-selected, transformed, or otherwise inexact Maps or
callables shall remain `E-COMPILER-UNSUPPORTED` before artifact publication.

The backend shall preserve the existing Topal-owned 16-byte Map header and
24-byte linked nodes. Each node shall retain the String pointer, ordinary i32
Function observation in its value slot, and next pointer. Capture snapshots
shall not be embedded in the header or nodes. `reject` shall diagnose duplicate
exact keys; `keep-first` shall retain the first callable/captures; `keep-last`
shall retain the last callable/captures at the first key's node position. Exact
lookup shall produce the ordinary `Optional Function`: a present key shall
attach only its retained callable facts to `Some`, while a missing key shall
produce `None`. Eventual application shall lower as one direct specialized
`fastcc` call. Display, DWARF, and the GDB printer shall retain source Map,
String-key, and Function observations without exposing compiler facts.

Every surviving Function value shall add its exact String key as a semantic
Map-value edge to its canonical capture path. Private parameters shall pass the
source Map pointer followed by resolved entry- and capture-ordered values.
Private results shall return the pointer followed by those values for one-time
caller extraction and remapping. This path shall compose with enclosing
Tuple/Record values and exact nested Function escape. Distinct keys and factory
calls shall retain independent snapshots. Function- or Generator-containing
capture state shall remain unsupported.

Tests shall cover nonempty named, symbolic, anonymous, and nested values; all
collision policies; present and missing exact-key lookup with complete Optional
decisions; count, emptiness, private parameter/result, package, Tuple, and
Record passage; independent factory snapshots; display; all interpreter modes
and reversible history; checked semantic Map-value paths and direct IR;
artifact-free empty/opaque-key/opaque-map/capture/repeated-identity rejection;
freestanding ELF/DWARF; full O0 GDB Map/callable frames; the shared corpus; and
separate interpreter/compiler resource baselines.

This shall add no closure/environment object, environment pointer, function
pointer, indirect call, callback, dispatch table, caller-frame lookup, foreign
dependency, C/C++ runtime, other-language standard library, public ABI, or
`topal-native/6` revision. Allocation shall remain the existing process-lifetime
source List nodes, Map nodes and header, and ordinary present Optional Function
box. Empty Map Function collection remains deferred until shared typed-empty
Map semantics are implemented. General Map Function equality/repeated identity,
dynamic lookup, operations that lose exact key/value facts, Function keys,
other key classifiers, opaque or dynamically selected values, persistent
storage, publication, recursive contained classifiers, and Function containment
in other unordered collections remain deferred. Future library metadata shall
retain canonical source-session/scope, collision policy, exact key set and
resolved order, source collection relationship, callable/declaration and
semantic key-based aggregate paths, ordered capture
identity/classifier/representation/lifetime, effects, Map/List representation
and ownership, and versioned target-adapter facts independently of headers,
node offsets, private observation tags, compiler names, LLVM types/symbols,
debug shadows, and physical placement. This realizes
`TOPAL-COMPILER-MAP-FUNCTION-001` for increments 3b2-b5ax, 6b2b2o, and 6c2m.

## TOPAL-COMP-ANONYMOUS-PRODUCT-001 — Private anonymous product patterns

The checked compiler model shall admit a flat positional product parameter
pattern in any position of a directly applied inferred anonymous Function. It
shall require one exact Tuple operand of matching arity for each product
pattern, bind fields in lexical source order, preserve the source parameter
arity, and compose with same-invocation immutable captures and non-capturing
anonymous Function parameters/results already admitted by the compiler.

The frontend shall evaluate the complete application operand exactly once.
Opaque Tuple locals and function results and the outer positional product of a
multi-parameter call shall use a compiler-private lexical value before field
projection. Direct Tuple constructions may retain sound field facts for body
checking, but no initializer or field expression may be replayed. A non-Tuple,
field-count mismatch, repeated binding outside
`TOPAL-COMP-ANONYMOUS-REPEATED-PATTERN-001`, or unsupported field representation
shall fail before anonymous-body or LLVM lowering.

LLVM definitions and calls shall use matching private `fastcc` signatures with
ordinary binding parameters and flattened product fields in lexical order,
followed by exact capture parameters. LLVM shall own physical x86-64 placement.
Full O0 DWARF/GDB shall expose non-discarded product fields as source-named
parameters, material captures, the anonymous frame, and callers. Tests shall
cover direct call-result materialization, bound captures, non-capturing
Function results, mixed product/scalar patterns, exact output, rejection
boundaries, once-only IR, artifact independence, DWARF validation, GDB
values/frames, all interpreter modes, reversible debugging, the shared corpus,
and separate resource baselines.

Recursive product patterns are governed by
`TOPAL-COMP-ANONYMOUS-NESTED-PATTERN-001`. This shall add no product-pattern
object, environment object, function pointer,
indirect call, closure or Function runtime, foreign dependency, C/C++ runtime,
other-language standard library, public aggregate or callable ABI, or
`topal-native/6` revision. Aggregate repeated-name patterns,
capturing Function boundaries outside
`TOPAL-COMP-FUNCTION-CAPTURE-PARAMETER-001` and
`TOPAL-COMP-FUNCTION-CAPTURE-RESULT-001`, other escape, aggregate containment
outside `TOPAL-COMP-FUNCTION-AGGREGATE-001`, publication, and library
metadata/adapters remain deferred. This realizes
`TOPAL-COMPILER-ANONYMOUS-PRODUCT-001`, `TOPAL-FUNCTION-ANONYMOUS-001`,
`TOPAL-TYPE-PRODUCT-001`, and `TOPAL-TYPE-CALL-001` for compiler increment
3b2-b5s.

## TOPAL-COMP-ANONYMOUS-NESTED-PATTERN-001 — Recursive anonymous product patterns

The syntax, interpreter, and checked compiler model shall admit a finite
positional product pattern recursively in any field of an inferred anonymous
Function parameter. Every product node shall require one exact Tuple with the
same arity and recursively visit binding/discard leaves depth-first from left
to right. A non-Tuple or mismatched field count at any depth shall fail before
body entry and before LLVM lowering.

The frontend shall evaluate the complete application operand once, retain one
compiler-private outer value when projection is required, and derive nested
fields only through recursive Tuple projections. It shall preserve top-level
source Function arity separately from the flattened leaf parameter list.
Existing repeated-name exact-identity guards and discard semantics shall apply
across all leaves. The pattern shall compose with immutable captures, mixed
top-level parameters, and capturing anonymous Function results already admitted
by their exact private boundaries.

LLVM definitions and calls shall use matching direct private `fastcc`
signatures containing admitted leaves in lexical order followed by captures,
with all physical x86-64 placement owned by LLVM. DWARF/GDB shall expose each
non-discarded leaf under its source name and omit compiler-only projection
storage. Tests shall cover opaque nested Tuple results, both nesting directions,
mixed parameters, captures, a capturing Function result, repeated names and
discard, non-Tuple and nested-arity rejection, checked structure, once-only
direct IR, exact interpreter modes and reversible history, freestanding
artifacts, full O0 GDB values/frames, the shared corpus, and separate resource
baselines.

This shall add no pattern object, allocation, environment object, function
pointer, indirect call, callback, closure or Function runtime, foreign
dependency, C/C++ runtime, other-language standard library, public aggregate
or callable ABI, or `topal-native/6` revision. Named-function header patterns,
aggregate containment outside `TOPAL-COMP-FUNCTION-AGGREGATE-001`, dynamic
escape/selection, publication, and library metadata/adapters remain deferred.
This realizes
`TOPAL-COMPILER-ANONYMOUS-NESTED-PATTERN-001`,
`TOPAL-COMPILER-ANONYMOUS-PRODUCT-001`, `TOPAL-FUNCTION-ANONYMOUS-001`,
`TOPAL-TYPE-PRODUCT-001`, `TOPAL-TYPE-MATCH-001`, and `TOPAL-TYPE-CALL-001` for
compiler increment 3b2-b5z.

## TOPAL-COMP-ANONYMOUS-REPEATED-PATTERN-001 — Exact repeated anonymous pattern names

The checked compiler model shall admit a repeated non-discard name across
ordinary scalar parameters and positional-product leaves of an inferred
anonymous Function, including recursive leaves under
`TOPAL-COMP-ANONYMOUS-NESTED-PATTERN-001`. The first occurrence shall be the
only body binding and debug parameter. Every later occurrence shall retain its
consumed private ABI
operand and record a lexical exact-identity guard against that first parameter.
Repeated `_` occurrences shall remain independent discards.

Both operands shall have one exact admitted scalar classifier: `Unit`,
`Completed`, `Effect`, `Type`, `Function`, `Boolean`, `Int`, `Nat`, one modular
type, `Rational`, `Comparison`, `ErrorCode`, one Enum type, `Character`, or
`String`. Identity shall use the existing same-representation direct comparison
without evidence forgetting, conversion, user Equality selection, canonical
equivalence, or approximation. A classifier mismatch shall fail in the checked
model. An opaque product-producing call and every field expression shall still
execute exactly once before any guard.

LLVM shall compare the private parameters in lexical guard order before the
body. Failure shall call a Topal-owned diagnostic helper which writes
`E-ANONYMOUS-PATTERN-IDENTITY` through the Linux syscall writer and exits 65.
Success shall continue directly into the anonymous body. Full O0 DWARF/GDB
shall show one source parameter for the first occurrence and the direct guarded
frame. Tests shall cover product and cross-parameter repetition, opaque product
materialization, Int, String, and Function identities, interpreter parity and
mismatch diagnostics, checked classifier rejection, direct guarded IR,
freestanding execution, GDB values/frames, the shared corpus, and separate
resource baselines.

This shall add no pattern object, matching table, function pointer, indirect
call, closure or Function runtime, foreign dependency, C/C++ runtime,
other-language standard library, public aggregate or callable ABI, or
`topal-native/6` revision. Aggregate repeated identity is governed by
`TOPAL-COMP-ANONYMOUS-REPEATED-AGGREGATE-001`. Ordinary named-function header
repetition, publication, and library
metadata/adapters remain deferred. This realizes
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`, `TOPAL-TYPE-MATCH-001`,
`TOPAL-FUNCTION-ANONYMOUS-001`, and `TOPAL-TYPE-CALL-001` for compiler increment
3b2-b5u.

## TOPAL-COMP-ANONYMOUS-REPEATED-AGGREGATE-001 — Exact repeated aggregate values

The checked compiler model shall admit a repeated anonymous-pattern name with
the same exact Tuple, Record, Optional, or List classifier when that classifier
already has both an admitted private function-boundary representation and
complete exact compiler equality. The initial boundary shall include recursive
equality-capable Tuple and Record values over admitted non-Function leaves;
`Optional Int`, `Optional Rational`, `Optional String`, and `Optional (Int,
String)`; `List Int`; and `List List (Int, String)`.

The first occurrence shall remain the sole source binding and DWARF parameter.
Every later occurrence shall retain its complete exact private parameter and a
lexically ordered pre-body guard. Tuple fields and canonical Record fields shall
compare recursively; Optional tags and present payloads shall compare exactly;
and Lists shall compare entries in order and require equal length. Lowering
shall reuse existing direct field comparisons and Topal-owned Optional/List
primitives without conversion, evidence forgetting, user Equality selection,
canonical equivalence, approximation, allocation identity, or inactive
representation data. A failed guard shall use the existing Topal-owned syscall
diagnostic and exit 65.

Tests shall cover opaque Tuple construction, canonical Records, Optional and
List values, exact interpreter modes and mismatch diagnostics, checked-model
metadata, structural guarded IR, native success and failure, freestanding
artifact properties, full O0 aggregate GDB observation, reversible debugging,
the shared corpus, and separate resource baselines. This shall add no generic
aggregate matcher, pattern table, allocation, callback, indirect dispatch,
foreign dependency, C/C++ runtime, other-language standard library, public
aggregate ABI, or `topal-native/6` revision. Sum identity is governed by
`TOPAL-COMP-ANONYMOUS-REPEATED-SUM-001`. Result, Range, Generator, refined,
authority-bearing, and Function-containing aggregate identity outside
`TOPAL-COMP-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001`; ordinary named-function
header repetition;
publication; and library metadata/adapters remain deferred. This realizes
`TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001`,
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`, `TOPAL-TYPE-MATCH-001`,
`TOPAL-FUNCTION-ANONYMOUS-001`, and `TOPAL-TYPE-CALL-001` for compiler increment
3b2-b5v.

## TOPAL-COMP-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001 — Exact capture-free Function aggregate values

The checked compiler model shall admit a repeated anonymous-pattern name with
the same exact recursively nested Tuple or Record classifier containing
Function leaves when every Function leaf retains one exact capture-free
callable identity and every other leaf has existing exact compiler equality.
It shall preserve complete recursive structural facts for the first and later
occurrences and reject missing or opaque callable facts before LLVM lowering.
Capture-bearing leaves shall be governed by
`TOPAL-COMP-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-AGGREGATE-001`.

The first occurrence shall remain the sole body binding and DWARF parameter.
Every later occurrence shall retain its complete recursive private aggregate
operand and a lexically ordered pre-body guard. Direct recursive Tuple/Record
comparison shall compare each Function observation field as i32 without using
that field for dispatch. Definitions and calls shall use matching exact private
`fastcc` aggregate signatures, with physical AMD64 placement left to LLVM's
qualified target data layout. Failure shall reuse the Topal-owned syscall
diagnostic and exit 65.

Tests shall cover named, symbolic, and non-capturing anonymous Function leaves,
nested Tuple/Record values, exact interpreter modes, native success and mismatch
failure, checked structural facts, direct guarded IR, freestanding artifact
properties, full O0 aggregate GDB observation,
reversible debugging, the shared corpus, and separate resource baselines. This
shall add no closure or environment object, allocation, generic matcher,
pattern table, function pointer, indirect call, callback, dispatch table,
foreign dependency, C/C++ runtime, other-language standard library, public
aggregate/callable ABI, or `topal-native/6` revision. Dynamic aggregate
selection, Function containment outside admitted Tuple/Record/Optional/Sum/Result/List/Array/Map paths, ordinary named-function
header repetition, publication, and library adapters remain deferred.
Compiled-library metadata
shall eventually encode canonical aggregate paths and stable
callable/representation identities without serializing private LLVM types,
observation tags, or target-specific argument placement. This realizes
`TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001`,
`TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001`,
`TOPAL-COMPILER-FUNCTION-AGGREGATE-001`,
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`, `TOPAL-TYPE-MATCH-001`,
`TOPAL-FUNCTION-ANONYMOUS-001`, and `TOPAL-TYPE-CALL-001` for compiler increment
3b2-b5ac.

## TOPAL-COMP-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001 — Exact captured anonymous Function values

The checked compiler model shall admit a repeated scalar Function pattern when
both operands retain the same anonymous source identity, the same ordered
represented capture schema, and exact compiler equality for every capture. It
shall canonicalize the anonymous source identity across private
specializations, retain both capture snapshots without re-evaluation, and
reject missing, inconsistent, or non-equality capture facts before LLVM.

The first Function occurrence shall remain the sole body binding and DWARF
parameter. The later occurrence shall retain its source observation field and
ordered captures as hidden parameters. Lowering shall compare the source field
first and the captures in order with existing exact direct comparisons, then
reuse the Topal-owned mismatch diagnostic and exit 65. The private definition
and call shall use matching exact `fastcc` prototypes, with physical AMD64
placement left to LLVM's qualified target data layout. Observation tags shall
remain non-dispatching and shall not become semantic library identities.

Tests shall cover same-source equal and unequal captures, canonical source
identity across specializations, non-equality capture rejection before artifact
publication, checked-model guard metadata, direct ordered IR, interpreter modes
and reversible history, the shared corpus, separate resource baselines,
freestanding ELF/DWARF properties, and full O0 GDB values/frames. This shall add
no closure or environment object, allocation, pattern table, function pointer,
indirect call, dispatch table, foreign dependency, C/C++ runtime,
other-language standard library, public callable ABI, or `topal-native/6`
revision. Captured named identity is governed by
`TOPAL-COMP-ANONYMOUS-REPEATED-CAPTURED-NAMED-FUNCTION-001`. Unsupported capture
classifiers, publication, and library adapters remain deferred. Future
compiled-library metadata shall encode stable callable
source identity plus the ordered capture schema/classifiers, equality
requirements, representation identity, lifetime/effects, and target adapter;
it shall not serialize private observation tags, hidden parameter names, LLVM
types, or target-specific placement. This realizes
`TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001`,
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`, `TOPAL-TYPE-MATCH-001`,
`TOPAL-FUNCTION-ANONYMOUS-001`, and `TOPAL-TYPE-CALL-001` for compiler increment
3b2-b5ad.

## TOPAL-COMP-ANONYMOUS-REPEATED-CAPTURED-NAMED-FUNCTION-001 — Exact captured named Function values

The checked compiler model shall admit a repeated scalar Function pattern when
both operands retain the same stable captured named declaration identity, the
same ordered represented capture schema, and exact compiler equality for every
required capture. The initial boundary shall be a non-escaping nested lexical
Function used within its defining invocation. It shall identify that callable
by source name and declaration identity across private specializations. When
declaration identities differ, the observation-field guard shall make the
values unequal without requiring capture transport. Missing declaration facts,
inconsistent required schemas, and required non-equality capture state shall
fail before LLVM or artifact publication.

The first Function occurrence shall remain the sole source binding and DWARF
parameter. When identities match, both capture snapshots shall follow source
parameters as deterministic hidden operands. Lowering shall compare the source
identity first and captures in retained order, then reuse the Topal-owned
mismatch diagnostic and exit 65. Matching private definitions and calls shall
use exact `fastcc` prototypes while LLVM owns physical AMD64 placement.
Observation fields shall remain non-dispatching and source offsets shall not
become semantic library identities.

Tests shall cover a captured nested Function repeated and directly invoked,
different captured nested declaration identities without unnecessary capture
equality, same-declaration non-equality capture rejection before artifact
publication, checked-model guards, direct ordered IR, interpreter modes and
reversible history, the shared corpus, separate resource baselines,
freestanding ELF/DWARF properties, and full O0 GDB values/frames. This shall add
no closure or environment object, allocation, pattern table, function pointer,
indirect call, dispatch table, foreign dependency, C/C++ runtime,
other-language standard library, public callable ABI, or `topal-native/6`
revision. Escaping nested Functions, unsupported capture classifiers, ordinary
named-function header repetition, publication, and library adapters remain
deferred. Future compiled-library metadata shall encode stable
declaration/source identity, ordered capture schemas and classifiers, semantic
equality requirements, representation identity, lifetime/effects, and target
adapters; it shall not serialize source offsets, private observation tags,
hidden operand names or layout, LLVM types, or target-specific placement. This
realizes
`TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-NAMED-FUNCTION-001`,
`TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001`,
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`, `TOPAL-FUNCTION-NESTED-001`,
`TOPAL-TYPE-MATCH-001`, and `TOPAL-TYPE-CALL-001` for compiler increment
3b2-b5af.

## TOPAL-COMP-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-AGGREGATE-001 — Exact captured Function aggregate values

The checked compiler model shall admit a repeated anonymous-pattern name with
the same exact recursively nested Tuple or Record classifier containing
captured Function leaves when both operands retain complete callable facts.
Ordinary fields shall retain their existing semantic-order structural guards.
When every corresponding Function leaf has the same stable named, symbolic,
nested, or canonical anonymous source identity, the model shall forward both
path-ordered capture snapshots, require the same capture schema at each path,
and require exact compiler equality for each represented capture classifier. If
any callable identity differs, its observation-field guard shall make the
aggregate unequal without requiring capture transport. Missing callable facts,
inconsistent schemas, and required non-equality capture state shall fail before
LLVM or artifact publication.

The first aggregate occurrence shall remain the sole source binding and DWARF
parameter. Its captures and, when required, the later occurrence's captures
shall follow source parameters as deterministic hidden operands ordered by
canonical aggregate path and capture order. Lowering shall compare ordinary
aggregate fields first and required capture pairs afterward, then reuse the
Topal-owned mismatch diagnostic and exit 65. Matching private definitions and
calls shall use exact `fastcc` prototypes while LLVM owns physical AMD64
placement. Observation fields shall remain non-dispatching.

Tests shall cover captured anonymous Function leaves in Record and nested
Tuple/Record values, captured nested named leaves, equal and unequal capture
payloads, callable-identity mismatch without unnecessary capture equality,
non-equality capture rejection before artifact publication when identities
match, checked-model path/capture guards, direct ordered IR, interpreter modes
and reversible history, the shared corpus, separate resource baselines,
freestanding ELF/DWARF properties, and full O0 GDB values/frames. This shall add
no closure or environment object, allocation, pattern table, function pointer,
indirect call, dispatch table, foreign dependency, C/C++ runtime,
other-language standard library, public aggregate/callable ABI, or
`topal-native/6` revision. Dynamic aggregate selection, Function containment
outside admitted Tuple/Record/Optional/Sum/Result/List/Array/Map paths, unsupported capture classifiers, ordinary named-function
header repetition, publication, and library adapters remain deferred. Future
compiled-library metadata shall encode canonical aggregate paths, stable
callable identities, ordered capture schemas and classifiers, semantic equality
requirements, representation identity, lifetime/effects, and target adapters;
it shall not serialize private observation tags, hidden operand names or layout,
LLVM types, or target-specific placement. This realizes
`TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-AGGREGATE-001`,
`TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001`,
`TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001`,
`TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`, `TOPAL-TYPE-MATCH-001`,
`TOPAL-FUNCTION-ANONYMOUS-001`, and `TOPAL-TYPE-CALL-001` for compiler increment
3b2-b5ae.

## TOPAL-COMP-ANONYMOUS-REPEATED-SUM-001 — Exact nominal Sum repeated identity

The checked compiler model shall admit a repeated anonymous-pattern name with
one exact nominal Union or positional Variant classifier when every possible
payload has existing exact compiler identity or recursively satisfies this
requirement. Tuple and Record fields may contain such a Sum recursively. The
model shall retain nominal identity and ordered alternative/payload schemas;
distinct declarations with structurally equal alternatives shall not match.
Function, Range, Result, refined, authority-bearing, and other payloads without
admitted exact identity shall fail before LLVM or artifact publication.

Lowering shall compare tags first. Unequal tags shall reach the existing Topal-
owned mismatch diagnostic and exit 65 without payload comparison. Equal tags
shall use LLVM `switch` to compare only the active alternative payload through
existing direct recursive comparisons and merge the result through an `i1`
`phi`. Inactive private fields shall neither be observed nor influence source
identity. This compiler-only guard shall not claim the general source Equality
capability for Sums.

Each operand shall retain the existing exact private aggregate in matching
`fastcc` caller and callee prototypes while LLVM owns physical AMD64 aggregate
and control-flow lowering. The first occurrence shall be the sole source
binding and DWARF parameter with its nominal Sum classifier and active value.
Tests shall cover labeled Union and positional Variant values; payload-free,
Int, String, Tuple, and nested Sum alternatives; same-value success; same-tag
payload mismatch; distinct-tag short-circuit;
unsupported payload rejection; checked-model facts; direct `switch`/`phi` IR;
interpreter modes and reversible history; the shared corpus; separate resource
baselines; freestanding ELF/DWARF properties; and full O0 GDB values/frames.

This shall add no generic matcher, Sum-equality runtime, pattern table,
allocation, callback, indirect dispatch, foreign dependency, C/C++ runtime,
other-language standard library, public aggregate ABI, or `topal-native/6`
revision. General derived Sum Equality is governed by
`TOPAL-COMP-SUM-EQUALITY-001`; recursive Sum declarations, Function-containing
Sums, persistent storage, publication, and library adapters remain deferred.
Future compiled-library metadata shall encode stable nominal
identity, positional/labeled form, ordered alternatives, payload schemas,
semantic identity/equality requirements, representation identity, and target
adapters; it shall not serialize private tags, inactive storage, LLVM types, or
target-specific placement. This realizes
`TOPAL-COMPILER-ANONYMOUS-REPEATED-SUM-001`,
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`, `TOPAL-TYPE-MATCH-001`,
`TOPAL-FUNCTION-ANONYMOUS-001`, and `TOPAL-TYPE-CALL-001` for compiler increment
3b2-b5ag.

## TOPAL-COMP-PACKAGED-OPERAND-001 — Closed scalar packaged operand

The checked compiler model shall admit exactly one packaged function operand
whose fields have admitted scalar classifiers. A full positional product shall
map to fields in declaration order. A labeled product shall be a
declaration-order prefix containing every required field, and omitted trailing
fields shall have closed defaults. The frontend shall preserve one-time
evaluation by retaining supplied declaration/source order followed by omitted
default order, adapt every field to its declared classifier, and distinguish
semantic missing/unknown/type failures from compiler-unsupported package
shapes.

The frontend shall normalize an admitted package into its source field list.
LLVM definitions and calls shall use the same exact flattened private `fastcc`
prototype, leaving physical x86-64 placement to LLVM. Full O0 DWARF/GDB shall
show each field as a source-named parameter with its value and the ordinary
function frame. Native tests shall cover the existing interpreter regression,
default and explicit/positional field supply, invalid and unsupported shapes,
exact IR, execution, artifact independence, and debugger observation.

The emitted IR shall use no `byval`, `sret`, `inalloca`, or `preallocated`
attribute and no package runtime or allocation. General label association is
governed by `TOPAL-COMP-PACKAGED-ASSOCIATION-ORDER-001`. Two or mixed packages
are governed by `TOPAL-COMP-COMPOUND-PACKAGED-OPERAND-001`. Exact capture-free
Tuple and Record fields are governed by
`TOPAL-COMP-STRUCTURED-PACKAGED-FIELD-001`; exact nominal Sum fields are
governed by `TOPAL-COMP-SUM-PACKAGED-FIELD-001`; exact direct Function fields
are governed by `TOPAL-COMP-FUNCTION-PACKAGED-FIELD-001`; exact
Function-containing Tuple and Record fields are governed by
`TOPAL-COMP-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`. Exact represented List,
Optional, Result, and Range fields are governed by
`TOPAL-COMP-CONTAINER-PACKAGED-FIELD-001`. Exact Array, Set, Bag, and Map fields
are governed by `TOPAL-COMP-COLLECTION-PACKAGED-FIELD-001`. Exact root and
root-alias Scope fields are governed by
`TOPAL-COMP-SCOPE-PACKAGED-FIELD-001`. Other non-scalar
fields,
nested package declarations, opaque package values, and invocation-dependent
defaults remain rejected until their complete evaluation, storage, and public
ABI rules are implemented. This shall add no foreign
dependency, C/C++ runtime, other-language standard library, public aggregate
ABI, or `topal-native/6` revision. This realizes
`TOPAL-COMPILER-PACKAGED-OPERAND-001`, `TOPAL-FUNCTION-PACKAGED-OPERAND-001`,
and `TOPAL-TYPE-CALL-001` for compiler increment 3b2-b5n.

## TOPAL-COMP-PACKAGED-ASSOCIATION-ORDER-001 — Label-based scalar package association

The checked compiler model shall extend the single admitted scalar package so a
labeled argument may supply each unique known field in any source order and may
omit a closed-defaulted field at any declaration position. It shall require
every nondefaulted field and reject unknown labels, duplicate labels, and
classifier mismatches before LLVM lowering or artifact publication.

The frontend shall retain every supplied expression exactly once in labeled
source order. If declaration order differs, it shall nest compiler-private
immutable bindings in source order and form the direct call from adapted local
references permuted into declaration order. Omitted closed defaults shall be
evaluated once in declaration order after supplied expressions. The private
bindings shall have no source-level or DWARF identity.

LLVM definitions and calls shall retain the existing matching flattened
private `fastcc` prototype in declaration order, with AMD64 register and stack
placement left to LLVM. GDB shall expose only the source fields, classifiers,
values, and function frame. Tests shall cover reordered supplied calls,
non-trailing omission, source-order function-call evaluation, positional
parity, unknown and duplicate rejection, exact IR, all interpreter modes,
reversible debugging, the shared corpus and separate resource baselines,
freestanding ELF/DWARF properties, and O0 GDB values/frames.

This shall add no package aggregate at the call boundary, `byval`, `sret`,
`inalloca`, `preallocated`, package runtime, allocation, foreign dependency,
C/C++ runtime, other-language standard library, public aggregate ABI, or
`topal-native/6` revision. Future compiled-library metadata shall encode stable
field identities, declaration order, default semantics and dependencies,
evaluation effects, representation identity, and target adapters independently
of compiler-private binding names, LLVM types, and physical placement. Two or
mixed packages are governed by `TOPAL-COMP-COMPOUND-PACKAGED-OPERAND-001`.
Exact capture-free Tuple and Record fields are governed by
`TOPAL-COMP-STRUCTURED-PACKAGED-FIELD-001`; exact nominal Sum fields are
governed by `TOPAL-COMP-SUM-PACKAGED-FIELD-001`; exact direct Function fields
are governed by `TOPAL-COMP-FUNCTION-PACKAGED-FIELD-001`; exact
Function-containing Tuple and Record fields are governed by
`TOPAL-COMP-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`. Exact represented List,
Optional, Result, and Range fields are governed by
`TOPAL-COMP-CONTAINER-PACKAGED-FIELD-001`. Exact Array, Set, Bag, and Map fields
are governed by `TOPAL-COMP-COLLECTION-PACKAGED-FIELD-001`. Exact root and
root-alias Scope fields are governed by
`TOPAL-COMP-SCOPE-PACKAGED-FIELD-001`. Other non-scalar
fields,
nested package declarations, opaque package values, invocation- or
capture-dependent defaults, and public package adapters remain deferred. This
realizes
`TOPAL-COMPILER-PACKAGED-ASSOCIATION-ORDER-001`,
`TOPAL-FUNCTION-PACKAGED-OPERAND-001`, and `TOPAL-TYPE-CALL-001` for compiler
increment 3b2-b5ai.

## TOPAL-COMP-COMPOUND-PACKAGED-OPERAND-001 — Two and mixed scalar packages

The checked compiler model shall admit a closed scalar package in either or
both of a function's two syntactic operand positions. An unpackaged operand
mixed with a package shall have an admitted non-callable scalar classifier.
Both source operands remain mandatory. Names shall be unique across all fields
and unpackaged operands. Unknown or duplicate labels, missing required fields,
classifier mismatches, and duplicate parameter names shall reject before LLVM
lowering or artifact publication.

The frontend shall retain every explicit ordinary operand and package field
exactly once in global source order: left operand before right operand and each
product's fields in source order. Omitted closed defaults shall execute after
all explicit values, in operand and field declaration order. The normalized
direct call shall receive ordinary operands and package fields in operand order,
with fields in declaration order. Compiler-private ordering bindings shall have
no source-level or DWARF identity.

LLVM definitions and calls shall use one matching flattened private `fastcc`
parameter per ordinary operand or package field, leaving physical AMD64
register and stack placement to LLVM. GDB shall expose only the source
parameters/fields, classifiers, values, and ordinary function frame. Tests
shall cover package/package, package/scalar, scalar/package, arbitrary label
order, closed defaults, positional parity, global source-order calls, invalid
labels and duplicate names, exact IR, all interpreter modes, reversible
debugging, the shared corpus and separate resource baselines, freestanding
ELF/DWARF properties, and O0 GDB values/frames.

This shall add no package aggregate at the call boundary, `byval`, `sret`,
`inalloca`, `preallocated`, package runtime, allocation, foreign dependency,
C/C++ runtime, other-language standard library, public aggregate ABI, or
`topal-native/6` revision. Future compiled-library metadata shall preserve
syntactic-operand partition/order beside stable field identities and order,
default semantics/dependencies, evaluation effects, representation identity,
and target adapters, independently of compiler-private names, LLVM types, and
physical placement. Exact capture-free Tuple and Record fields are governed by
`TOPAL-COMP-STRUCTURED-PACKAGED-FIELD-001`; exact nominal Sum fields are
governed by `TOPAL-COMP-SUM-PACKAGED-FIELD-001`; exact direct Function fields
are governed by `TOPAL-COMP-FUNCTION-PACKAGED-FIELD-001`; exact
Function-containing Tuple and Record fields are governed by
`TOPAL-COMP-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`. Exact represented List,
Optional, Result, and Range fields are governed by
`TOPAL-COMP-CONTAINER-PACKAGED-FIELD-001`. Exact Array, Set, Bag, and Map fields
are governed by `TOPAL-COMP-COLLECTION-PACKAGED-FIELD-001`. Exact root and
root-alias Scope fields are governed by
`TOPAL-COMP-SCOPE-PACKAGED-FIELD-001`. Other Function-containing aggregates,
other non-scalar fields, nested package
declarations, opaque compound package values, invocation- or capture-dependent
defaults, recursive compound signatures, and public package adapters remain
deferred. This realizes
`TOPAL-COMPILER-COMPOUND-PACKAGED-OPERAND-001`,
`TOPAL-FUNCTION-PACKAGED-OPERAND-001`, and `TOPAL-TYPE-CALL-001` for compiler
increment 3b2-b5aj.

## TOPAL-COMP-STRUCTURED-PACKAGED-FIELD-001 — Exact Tuple and Record package fields

The checked compiler model shall admit exact capture-free Tuple and Record
classifiers as fields of one- or two-operand packages when their recursively
composed leaf classifiers are already supported by the private function ABI.
Supplied values and closed defaults shall undergo the same exact structural
adaptation as ordinary private aggregate parameters. Function-containing Tuple
and Record fields are governed by
`TOPAL-COMP-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`. Unsupported leaves,
mismatched shapes, and non-closed defaults shall reject before LLVM lowering or
artifact publication.

The frontend shall retain each explicit structured field once in the source
order established by the package rules, evaluate structured closed defaults
after explicit values in operand/field declaration order, and pass each complete
field in operand/field declaration order. A structured field shall remain one
source field and one exact private parameter; its internal components shall not
become package fields. Compiler-private ordering bindings shall have no source
or DWARF identity.

LLVM definitions and calls shall use matching exact aggregate `fastcc`
parameters and leave physical AMD64 placement to LLVM. GDB shall expose the
source field name, complete recursively structured classifier/value, and
ordinary function frame. Tests shall cover one-package and mixed two-operand
calls, Tuple and Record fields, reordered once-only function calls, a closed
Record default, positional parity, opaque and unsupported-field rejection,
exact aggregate IR, all interpreter modes, reversible debugging, the shared
corpus and separate resource baselines, freestanding ELF/DWARF properties, and
O0 GDB values/frames.

This shall add no package-level aggregate, `byval`, `sret`, `inalloca`,
`preallocated`, package runtime, allocation beyond an existing value
representation, foreign dependency, C/C++ runtime, other-language standard
library, public aggregate ABI, or `topal-native/6` revision. Future
compiled-library metadata shall preserve complete canonical structural
classifiers and representation identities beside operand/field/default/effect
semantics and target adapters. Exact Function-containing Tuple and Record fields
are governed by `TOPAL-COMP-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`. Nested
package declarations, opaque whole-package values, dependent defaults,
recursive structured package signatures, public adapters, and other unsupported
non-scalar fields remain deferred. Exact nominal Sum fields are governed by
`TOPAL-COMP-SUM-PACKAGED-FIELD-001`; exact direct Function fields are governed
by `TOPAL-COMP-FUNCTION-PACKAGED-FIELD-001`. This realizes
`TOPAL-COMPILER-STRUCTURED-PACKAGED-FIELD-001`,
`TOPAL-FUNCTION-PACKAGED-OPERAND-001`, and `TOPAL-TYPE-CALL-001` for compiler
increment 3b2-b5ak.

## TOPAL-COMP-SUM-PACKAGED-FIELD-001 — Exact nominal Sum package fields

The checked compiler model shall admit an exact nominal Sum classifier as a
field of one- or two-operand packages when every alternative payload is already
supported by the private Sum function ABI and contains no Function value or
capture environment. Supplied values and closed defaults shall retain exact
nominal identity, tag, and active payload through ordinary classifier
adaptation. Unsupported payloads, nominal mismatches, and non-closed defaults
shall reject before LLVM lowering or artifact publication.

The frontend shall retain each explicit Sum field once in package source order,
evaluate closed Sum defaults afterward in operand/field declaration order, and
pass each complete Sum in operand/field declaration order. A Sum field shall
remain one source field and one exact private tag-plus-payload parameter;
alternatives and payload components shall not become package fields.
Compiler-private ordering bindings shall have no source or DWARF identity.

LLVM definitions and calls shall use matching exact Sum `fastcc` parameters and
leave physical AMD64 placement to LLVM. GDB shall expose the source field name,
nominal classifier, active alternative/payload value, and ordinary function
frame. Tests shall cover one-package and mixed two-operand calls, reordered
once-only calls, a closed Sum default, positional parity, opaque and
unsupported-Function-payload rejection, exact Sum IR, all interpreter modes, reversible
debugging, the shared corpus and separate resource baselines, freestanding
ELF/DWARF properties, and O0 GDB values/frames.

This shall add no package-level aggregate, payload decomposition, `byval`,
`sret`, `inalloca`, `preallocated`, package runtime, allocation beyond the
existing Sum representation, foreign dependency, C/C++ runtime, other-language
standard library, public aggregate ABI, or `topal-native/6` revision. Future
compiled-library metadata shall preserve canonical nominal identity, complete
alternatives/payload classifiers, representation identity,
operand/field/default/effect semantics, and target adapters. Nested package
declarations, opaque whole-package values, Function-containing Sum payloads
outside `TOPAL-COMP-SUM-FUNCTION-001`, dependent defaults, recursive Sum
package signatures, public adapters, and other unsupported non-scalar fields
remain deferred. This realizes
`TOPAL-COMPILER-SUM-PACKAGED-FIELD-001`,
`TOPAL-FUNCTION-PACKAGED-OPERAND-001`, and `TOPAL-TYPE-CALL-001` for compiler
increment 3b2-b5al.

## TOPAL-COMP-FUNCTION-PACKAGED-FIELD-001 — Exact direct Function package fields

The checked compiler model shall admit an exact Function value as a direct
field of a one- or two-operand package when its callable identity and any
represented immutable capture snapshot are already admitted by the private
Function parameter ABI. Supplied values and closed defaults shall retain the
same named, symbolic, anonymous, or non-escaping nested callable facts used by
an ordinary Function parameter. Opaque, dynamically selected, escaping, or
otherwise unrepresentable callable values shall reject before LLVM lowering or
artifact publication.

The frontend shall preserve each explicit Function initializer once in package
source order. A compiler-private ordering binding shall retain exact callable
and capture facts beside the runtime Function observation tag, and the direct
call shall permute that retained value into operand/field declaration order.
One source Function field shall remain one source parameter; any deterministic
hidden capture parameters required by the existing specialized callable
boundary shall not become package fields or source parameters.

LLVM definitions and calls shall use matching private `fastcc` Function-tag and
capture parameters and leave physical AMD64 placement to LLVM. Callee
application shall remain a direct specialized call without function-pointer
dispatch. GDB shall expose the source field identity/value and ordinary frame,
while compiler-private bindings and hidden capture transport remain absent from
source debugging. Tests shall cover reordered once-only Function-returning and
scalar calls, a closed symbolic default, positional parity, mixed operands,
captured callable forwarding, opaque-package rejection, direct IR, all
interpreter modes, reversible debugging, the shared corpus and separate
resource baselines, freestanding ELF/DWARF properties, and O0 GDB values/frames.

This shall add no package aggregate, function pointer, indirect call, closure
allocation/runtime, `byval`, `sret`, `inalloca`, `preallocated`, foreign
dependency, C/C++ runtime, other-language standard library, public callable ABI,
or `topal-native/6` revision. Future compiled-library metadata shall preserve
callable source/declaration identity, overload set, capture schema and equality,
representation/lifetime/effect semantics, operand/field/default semantics, and
target adapters. Exact Function-containing Tuple and Record fields are governed
by `TOPAL-COMP-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`. Function containment in
other aggregates, dynamic escape/selection, nested package declarations, opaque
whole-package values, dependent defaults, recursive callable package
signatures, public adapters, and other unsupported fields remain deferred. This
realizes
`TOPAL-COMPILER-FUNCTION-PACKAGED-FIELD-001`,
`TOPAL-FUNCTION-PACKAGED-OPERAND-001`, and `TOPAL-TYPE-CALL-001` for compiler
increment 3b2-b5am.

## TOPAL-COMP-FUNCTION-AGGREGATE-PACKAGED-FIELD-001 — Exact Function aggregate package fields

The checked compiler model shall admit an exact Tuple or Record containing one
or more Function leaves as a complete package field when its full structural
classifier, callable identities, and immutable capture snapshots are already
admitted by the private Function aggregate parameter boundary. Every Function
leaf shall retain exact named, symbolic, anonymous, or non-escaping nested
callable facts. Missing, opaque, dynamically selected, escaping, or otherwise
unrepresentable callable facts shall reject before LLVM lowering or artifact
publication.

For every compiler-private package-ordering binding, the frontend shall retain
the aggregate's complete recursive structural fact tree, canonical Function
paths, callable identities, and capture facts beside the runtime aggregate.
Each explicit initializer shall execute once in package source order. Closed
aggregate defaults shall execute afterward in operand/field declaration order,
and each complete aggregate shall remain one source field and one private
aggregate parameter rather than being decomposed into package fields.

LLVM definitions and calls shall use the existing exact target-derived
aggregate type followed by deterministic canonical-path-ordered hidden capture
parameters under matching private `fastcc` prototypes. LLVM shall own physical
AMD64 aggregate classification and placement. Callee application shall remain
a direct specialization without function-pointer dispatch. GDB shall expose
the one complete source Tuple or Record field and ordinary frame while private
ordering bindings and hidden captures remain absent from source debugging.

Tests shall cover reordered once-only scalar and Function-aggregate-producing
calls, capture-free Record and capture-bearing Tuple fields, a closed symbolic
Function-aggregate default, exact output, opaque whole-package artifact-free
rejection, recursive checked facts, exact direct IR, all interpreter modes,
reversible debugging, the shared corpus and separate resource baselines,
freestanding ELF/DWARF properties, and O0 GDB aggregate values/frames.

This shall add no package-level aggregate, member decomposition, function
pointer, indirect call, closure allocation/runtime, `byval`, `sret`, `inalloca`,
`preallocated`, foreign dependency, C/C++ runtime, other-language standard
library, public callable/aggregate ABI, or `topal-native/6` revision. Future
compiled-library metadata shall preserve complete canonical aggregate
classifiers and representation identities, Function-leaf paths and
source/declaration identities, overload sets, capture schemas and equality,
lifetime/effect semantics, operand/field/default semantics, and target adapters
independently of private binding names, LLVM types, tags, and physical
placement. Optional Function fields are governed by
`TOPAL-COMP-OPTIONAL-FUNCTION-001`; exact nominal Sum Function fields are
governed by `TOPAL-COMP-SUM-FUNCTION-001`; exact arithmetic Result Function
fields are governed by `TOPAL-COMP-RESULT-FUNCTION-001`; exact finite List
Function fields are governed by `TOPAL-COMP-LIST-FUNCTION-001`; exact fixed-size
Array Function fields are governed by `TOPAL-COMP-ARRAY-FUNCTION-001`; exact
nonempty String-keyed Map Function fields are governed by
`TOPAL-COMP-MAP-FUNCTION-001`. Function
containment in other aggregates; dynamic escape/selection; dependent defaults;
nested package declarations;
recursive callable package signatures; persistent storage; publication; and
public adapters remain deferred. This realizes
`TOPAL-COMPILER-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`,
`TOPAL-COMPILER-FUNCTION-AGGREGATE-001`,
`TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`,
`TOPAL-FUNCTION-PACKAGED-OPERAND-001`, and `TOPAL-TYPE-CALL-001` for compiler
increment 3b2-b5an.

## TOPAL-COMP-CONTAINER-PACKAGED-FIELD-001 — Exact represented container package fields

The checked compiler model shall admit exact List, Optional, Result, and Range
classifiers as complete fields of one- or two-operand packages when each exact
classifier is already admitted by its private function parameter boundary. The
contained element, value, success/error-domain, or endpoint classifiers shall
remain within that existing represented boundary and shall contain no Function
value or capture environment under this requirement. Exact `Optional Function`
fields are instead governed by `TOPAL-COMP-OPTIONAL-FUNCTION-001`; exact
arithmetic `Result Function` fields are governed by
`TOPAL-COMP-RESULT-FUNCTION-001`; exact finite `List Function` fields are
governed by `TOPAL-COMP-LIST-FUNCTION-001`; exact fixed-size `Array Function`
fields are governed by `TOPAL-COMP-ARRAY-FUNCTION-001`; exact nonempty
`Map (String, Function)` fields are governed by
`TOPAL-COMP-MAP-FUNCTION-001`. Unsupported contained
classifiers and mismatched container classifiers shall reject before
LLVM lowering or artifact publication.

The frontend shall retain every explicit represented-container initializer
once in package source order with its exact checked classifier. A closed
default whose analyzed value already has the exact declared container
classifier shall execute afterward in operand/field declaration order. Direct
calls shall pass each complete source field in operand/field declaration order.
Implicit context-dependent construction or lifting of a differently classified
default, including a bare success value into Result, remains rejected in this
increment.

LLVM definitions and calls shall use one matching private pointer-carrier
`fastcc` parameter per source field and leave physical AMD64 placement to LLVM.
Contained values shall not become package fields. Package normalization shall
add no allocation or alter the existing process-lifetime ownership of List
nodes. GDB shall expose the complete List, Optional, Result, and Range source
fields and ordinary frame while private ordering bindings remain absent from
source debugging.

Tests shall cover all four represented container families, reordered once-only
container-producing calls, one exact closed Optional default, exact output,
unsupported-field artifact rejection, checked classifier retention, matching
direct pointer-carrier IR, all interpreter modes, reversible debugging, the
shared corpus and separate resource baselines, freestanding ELF/DWARF
properties, and O0 GDB values/frames.

This shall add no package aggregate, contained-value decomposition, `byval`,
`sret`, `inalloca`, `preallocated`, package runtime, allocation beyond existing
container construction, foreign dependency, C/C++ runtime, other-language
standard library, public container ABI, or `topal-native/6` revision. Future
compiled-library metadata shall preserve the canonical generic constructor,
contained classifier or Result error domain, Range endpoint classifier,
representation/lifetime/default/effect semantics, stable operand/field
identities and order, and target adapters independently of compiler-private
binding names, LLVM pointer types, and physical placement. Function-containing
containers outside exact `Optional Function`, arithmetic `Result Function`,
finite `List Function`, and fixed-size `Array Function`, unsupported List
elements and container payloads, context-dependent defaults, nested package declarations,
opaque whole-package values, recursive package signatures, persistent storage,
publication, and public adapters remain deferred. This realizes
`TOPAL-COMPILER-CONTAINER-PACKAGED-FIELD-001`,
`TOPAL-FUNCTION-PACKAGED-OPERAND-001`, and `TOPAL-TYPE-CALL-001` for compiler
increment 3b2-b5ao.

## TOPAL-COMP-COLLECTION-PACKAGED-FIELD-001 — Exact represented collection package fields

The checked compiler model shall recognize exact `Array (N, Int)`, `Set Int`,
`Bag Int`, and `Map (String, Int)` private function parameter and result
classifiers, with a statically parsed nonnegative Array extent, and admit those
values as complete fields of one- or two-operand packages. Other element,
key, or value classifiers and mismatched collection classifiers shall reject
before LLVM lowering or artifact publication rather than being accepted from
their pointer carrier alone. Exact `Array (N, Function)` fields are instead
governed by `TOPAL-COMP-ARRAY-FUNCTION-001`; exact nonempty
`Map (String, Function)` fields are governed by
`TOPAL-COMP-MAP-FUNCTION-001`.

The frontend shall retain every explicit collection-producing initializer once
in package source order with its exact constructor and extent/element/key/value
classifier facts. Direct calls shall pass each complete source field in
operand/field declaration order. Closed defaults remain governed by the
existing exact checked-default rule; constructions not already analyzable as
the declared collection classifier shall remain rejected.

LLVM definitions and calls shall use one matching private pointer-carrier
`fastcc` parameter per source field and leave physical AMD64 placement to LLVM.
Entries, keys, values, multiplicities, and collision state shall not become
package fields. Package normalization shall add no allocation or alter the
existing process-lifetime ownership of collection construction. GDB shall
expose the complete Array, Set, Bag, and Map source fields and ordinary frame
while private ordering bindings remain absent from source debugging.

Tests shall cover all four exact collection families, reordered once-only
collection-producing calls, positional parity, exact output, unsupported or
mismatched classifier artifact rejection, checked classifier retention,
matching direct pointer-carrier IR, all interpreter modes, reversible
debugging, the shared corpus and separate resource baselines, freestanding
ELF/DWARF properties, and O0 GDB values/frames.

This shall add no package aggregate, entry decomposition, `byval`, `sret`,
`inalloca`, `preallocated`, package runtime, allocation beyond existing
collection construction, foreign dependency, C/C++ runtime, other-language
standard library, public collection ABI, or `topal-native/6` revision. Future
compiled-library metadata shall preserve the canonical constructor; Array
extent; element or key/value classifiers; ordering, uniqueness, multiplicity,
and collision semantics; representation/lifetime/default/effect semantics;
stable operand/field identities and order; and target adapters independently
of compiler-private binding names, LLVM pointer types, and physical placement.
Function-containing collections outside exact `Array (N, Function)` and
nonempty exact `Map (String, Function)`, other generic specializations,
context-dependent or otherwise unanalyzable defaults, nested package
declarations, opaque whole-package values, recursive package signatures,
persistent collection storage, publication, and public adapters remain
deferred. This realizes `TOPAL-COMPILER-COLLECTION-PACKAGED-FIELD-001`,
`TOPAL-FUNCTION-PACKAGED-OPERAND-001`, and `TOPAL-TYPE-CALL-001` for compiler
increment 3b2-b5ap.

## TOPAL-COMP-SCOPE-PACKAGED-FIELD-001 — Exact root Scope package fields

The checked compiler model shall admit the live source-root namespace and an
exact retained root alias as a complete `Scope` field of one- or two-operand
packages. It shall retain the namespace identity, binding-position declaration
snapshot, function overload order, and represented data facts through any
compiler-private source-order binding. A closed default of `root` shall be
admitted. Opaque or computed Scope values, nested or non-root namespaces, and a
live `root` package field formed inside a compiled function shall reject before
LLVM lowering or artifact publication.

The frontend shall evaluate every explicit field initializer once in package
source order and pass source fields in operand/field declaration order. It
shall reuse `TOPAL-COMP-NAMESPACE-BOUNDARY-001`: the private call shall pass one
explicit sealed `i32` Scope observation followed by the exact represented
immutable namespace data values in deterministic existing order. Qualified
function selection shall remain static and direct. LLVM shall own physical
AMD64 placement, and package normalization shall add no allocation or alter
namespace identity, snapshot, or lifetime.

Full O0 DWARF/GDB shall expose the source Scope field and namespace observation,
each material represented data argument, and the ordinary function frame while
compiler-private ordering bindings remain absent. Tests shall cover labeled
reordering with once-only evaluation, positional parity, a closed `root`
default, exact output, function-body live-root artifact rejection, checked
namespace-fact retention, direct matching IR, all interpreter modes, reversible
history, the shared corpus and separate resource baselines, freestanding
ELF/DWARF properties, and GDB Scope/data values and frames.

This shall add no package aggregate, namespace table, dynamic lookup, function
pointer, indirect call, environment or package allocation, `byval`, `sret`,
`inalloca`, `preallocated`, foreign dependency, C/C++ runtime, other-language
standard library, public Scope ABI, or `topal-native/6` revision. Future
compiled-library metadata shall preserve namespace identity and snapshot
position; member visibility and declaration order; complete function overload,
generator, and represented-data schemas; hidden data classifier/order,
lifetime, and effect facts; package operand/field identity and order; default
semantics and dependencies; and versioned target adapters independently of the
private `i32` tag, hidden parameter names, LLVM types, symbols, and physical
placement. Opaque, computed, nested, non-root, external, escaping, result, and
public/library Scope environments remain deferred. This realizes
`TOPAL-COMPILER-SCOPE-PACKAGED-FIELD-001`,
`TOPAL-COMPILER-NAMESPACE-BOUNDARY-001`,
`TOPAL-FUNCTION-PACKAGED-OPERAND-001`, and `TOPAL-TYPE-CALL-001` for compiler
increment 3b2-b5aq.

## TOPAL-COMP-CONTEXT-CAPTURE-001 — Private defining-context capture

The checked compiler model shall admit `@ member` in a root function called
directly from the source entry frame when `member` is an admitted scalar root
binding declared before the function. Resolution shall use only the function's
immutable defining context: later root declarations and same-named caller or
lexical bindings shall not participate. The call shall reuse the already
evaluated root compiler value and shall not replay its initializer.

The frontend shall detect referenced context members and append them in root
declaration order as explicit private capture arguments and parameters. The
capture shall retain its exact checked classifier and static facts. LLVM shall
receive the same exact private `fastcc` prototype at definition and call sites
and own physical x86-64 placement. Full O0 DWARF/GDB shall expose `@ member`, its
source type and value, and the surrounding ordinary source function frame.
Native tests shall cover exact output, lexical shadow isolation, declaration
order, invalid context use, IR, artifact independence, and debugger observation.

This increment shall add no global context storage, lookup table, closure
allocation/runtime, function pointer, indirect call, foreign dependency, C/C++
runtime, other-language standard library, public closure ABI, or
`topal-native/6` revision. Aggregate captures beyond
`TOPAL-COMP-AGGREGATE-ENVIRONMENT-001`, Scope and Function captures,
anonymous/escaping closures, qualified root data not admitted by
`TOPAL-COMP-FUNCTION-ROOT-DATA-001`, and public/library contexts remain rejected.
Exact scalar forwarding is governed by
`TOPAL-COMP-CONTEXT-CAPTURE-FORWARD-001`. This realizes
`TOPAL-COMPILER-CONTEXT-CAPTURE-001` and `TOPAL-CONTEXT-SELECT-001` for compiler
increment 6c1.

## TOPAL-COMP-CONTEXT-CAPTURE-FORWARD-001 — Private defining-context forwarding

For a finite acyclic chain of statically named ordinary root functions reached
from the source entry frame, the checked model shall compute transitive exact
`@ member` selections before instantiating the outer function. Each selected
value shall come from the immutable context snapshot of the function containing
that direct selection, with declaration-position filtering applied there. The
frontend shall append every required supported private machine value to each
intermediate function's hidden capture group in root declaration order and
forward the same already-evaluated value at every direct call edge. Same-named
caller bindings, parameters, and locals shall remain isolated in every frame.

The Linux x86-64 backend shall emit matching exact `fastcc` definitions and
calls and leave physical placement to LLVM. Full O0 DWARF/GDB shall expose all
explicit and forwarded parameters accurately in the active function and every
suspended caller frame. Target-aligned debug-only stack shadows may preserve
call-clobbered values without adding semantic storage. Tests shall cover a
three-frame shared regression, interpreter modes, reversible history, checked
capture order and call arguments, unresolved overload-selection rejection,
exact direct IR, artifact-free failure, freestanding ELF/DWARF, every GDB frame,
the shared corpus, and separate resource baselines.

This shall add no global context storage, namespace/capture/environment table,
initializer replay, lookup, allocation, function pointer, indirect call,
foreign dependency, C/C++ runtime, other-language standard library, public ABI,
or `topal-native/6` revision. Overload forwarding beyond
`TOPAL-COMP-OVERLOAD-ENVIRONMENT-001`, recursive forwarding beyond
`TOPAL-COMP-RECURSIVE-SCALAR-ENVIRONMENT-001`, local named Function forwarding
beyond `TOPAL-COMP-LOCAL-FUNCTION-ENVIRONMENT-001`, anonymous functions,
aggregate environments beyond
`TOPAL-COMP-AGGREGATE-ENVIRONMENT-001`, otherwise unsupported context members,
escape, and public/library environments remain deferred. Future
compiled-library metadata shall preserve canonical
defining-context instance and source-session identity, every selection and call
edge, callee identity/overload, member stable identity, captured declaration
position, visibility/declaration order, classifier/semantic representation,
capture order/lifetime/effects, and a versioned target adapter independently of
private names, LLVM types/symbols, debug shadows, and physical placement. This
realizes `TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001` and
`TOPAL-CONTEXT-SELECT-001` for increment 6c2a.

## TOPAL-COMP-RECURSIVE-SCALAR-ENVIRONMENT-001 — Proof-backed recursive scalar environments

The checked model shall permit exact scalar hidden captures on a direct or
mutual recursive edge only after an existing explicit-measure, Int, Nat, or
mutual recursion proof independently admits that edge. Capture discovery shall
not establish termination. Before closing a cycle, the frontend shall compute
the transitive union of required `@ member` and `root member` values for every
graph member, append them in established declaration order, and use the current
hidden parameters as every recursive call argument. The source entry edge alone
shall supply the immutable defining-context snapshot and live-root call-position
snapshot.

Every cycle member and call shall use the same exact private `fastcc` prototype
and already-reserved symbol, with LLVM owning x86-64 placement. Definitions
shall remain `noinline` without claiming `norecurse`. Full O0 DWARF/GDB shall
expose the explicit parameter and each context/root capture in the active frame
and every suspended direct or mutual frame; aligned debug-only shadows may
retain call-clobbered values without adding semantic state. Tests shall cover
direct, mutual, and explicit-measure checked graphs, transitive captures used by
different cycle members, unproven artifact-free rejection, exact cyclic IR,
interpreter modes, reversible history, freestanding ELF/DWARF, the shared
corpus, resource baselines, and every recursive GDB frame.

This shall add no global root/context storage, environment/cycle table, lookup,
initializer replay, allocation, dispatcher, function pointer, indirect call,
foreign dependency, C/C++ runtime, other-language standard library, public ABI,
or `topal-native/6` revision. Unproven, incomplete, or mixed-proof cycles;
overload selection beyond `TOPAL-COMP-OVERLOAD-ENVIRONMENT-001`; unsupported
representations; local named Function recursion beyond
`TOPAL-COMP-LOCAL-FUNCTION-ENVIRONMENT-001`; anonymous/nested/escaping
recursion; and public/library recursive environments remain deferred. Future
library metadata shall retain
recursion graph/member identity, proof rule/evidence, every ordered call and
capture edge, canonical context/root instance, member identity and declaration
position, classifier/semantic representation, capture order/lifetime/effects,
and a versioned target adapter independently of private symbols, LLVM types,
debug shadows, and physical placement. This realizes
`TOPAL-COMPILER-RECURSIVE-SCALAR-ENVIRONMENT-001` for increments 6b2b2d and
6c2b.

## TOPAL-COMP-AGGREGATE-ENVIRONMENT-001 — Private represented aggregate environments

The checked compiler model shall admit exact Tuple, labeled Record, and nominal
Sum values as hidden defining-context and live-root captures when every
component has an already-admitted private representation and the complete
aggregate contains no Function value. The same admission shall apply at direct
entry calls, through finite acyclic statically named forwarding chains, and on
every direct or mutual recursive edge independently admitted by the existing
termination proofs. Capture discovery shall not establish termination.

The entry edge shall pass each already-evaluated immutable aggregate, and every
forwarding or recursive edge shall rebuild and pass that complete semantic
value unchanged. Tuple positions, Record source field order and labels, and Sum
nominal identity, active alternative, and payload shall remain exact. The Linux
x86-64 backend shall emit matching compiler-private by-value LLVM aggregate
types under `fastcc` without `byval`, `sret`, `inalloca`, or `preallocated`
directives. LLVM shall own physical AMD64 placement and future target adapters.

Full O0 DWARF/GDB shall expose each aggregate by its source capture name and
classifier in active and suspended forwarding or recursive frames. Aligned
debug-only shadows may retain call-clobbered aggregates without semantic
storage. Tests shall cover Tuple, Record, and Sum context/root values; direct,
acyclic, and proven recursive edges; checked-model capture types and cycle
union; exact output and private IR; Function-aggregate artifact-free rejection;
interpreter modes; reversible history; freestanding ELF/DWARF; GDB values in
active and suspended frames; the shared corpus; and separate resource
baselines.

This shall add no global root/context storage, aggregate environment object,
environment/namespace table, lookup, replay, allocation, dispatcher, function
pointer, indirect call, foreign dependency, C/C++ runtime, other-language
standard library, public ABI, or `topal-native/6` revision. Function-bearing
aggregates; Scope, Generator, constraint, evidence, static-only, opaque, or
otherwise unsupported representations; overload selection beyond
`TOPAL-COMP-OVERLOAD-ENVIRONMENT-001`; local named Function forwarding beyond
`TOPAL-COMP-LOCAL-FUNCTION-ENVIRONMENT-001`; anonymous/escaping functions; and
public/library aggregate environments remain deferred. Future library metadata
shall retain
canonical context/root instance and source session, every selection/call edge,
callee identity/overload, member identity/declaration position, complete
semantic classifier and recursive component structure, Tuple/Record ordering
and labels, Sum identity/alternative/payload, capture order/lifetime/effects,
and a versioned target adapter independently of private capture names, LLVM
aggregate types/symbols, debug shadows, and physical placement. This realizes
`TOPAL-COMPILER-AGGREGATE-ENVIRONMENT-001` for increments 6b2b2e and 6c2c.

## TOPAL-COMP-OVERLOAD-ENVIRONMENT-001 — Exact overload-selected private environments

For an ordinary statically named overloaded call, the checked model shall use
only the source-ordered declaration selected by the ordinary call rules when
selection is known before instantiation from closed Unit, Boolean, Int,
Rational, or String literals, explicit parameter classifiers, exact
Tuple/Record products of those values, or classifier-preserving `+`, `-`, or
`*` over equal admitted classifiers. Capture discovery and the eventual direct
call shall retain the same declaration identity. This shall compose with direct
entry, finite acyclic and cross-overload chains, represented aggregate
environments, and independently proven direct/mutual recursion.

Every overload shall retain its own exact context/root capture vector. An
unselected overload shall neither add nor remove a hidden parameter. Selected
cross-overload and recursive edges shall forward the current exact values in
the established order through matching private `fastcc` prototypes, with LLVM
owning x86-64 placement. Capture discovery shall not establish termination.

When selection could change with retained value facts—including `Int` to
`Nat`, `String` to `Character`, or exact `Rational` narrowing—or depends on a
packaged/defaulted/qualified, locally inferred, higher-order, anonymous,
nested, dynamic, or otherwise unresolved call, the frontend shall reject before
artifact publication whenever any candidate carries an environment. It shall
not union candidate capture sets or invent a provisional overload-call ABI. A
first-class retained overload set admitted by
`TOPAL-COMP-FUNCTION-ENVIRONMENT-BOUNDARY-001` shall instead transport its
complete deduplicated semantic environment while retaining declaration-specific
capture schemas separately, and its selected direct call shall still receive
only the selected declaration's vector. An exact retained named alias admitted
by `TOPAL-COMP-LOCAL-FUNCTION-ENVIRONMENT-001` shall use the
same admitted evidence and is not otherwise unresolved here. Tests shall cover
distinct overload capture sets, unselected-capture isolation, closed literal
and explicit-parameter selection, exact aggregate values, acyclic and
cross-overload forwarding, proven recursion, checked capture vectors, exact
output/IR, value-fact-dependent artifact-free rejection, interpreter modes,
reversible history, freestanding ELF/DWARF, selected active/suspended GDB
frames, the shared corpus, and separate resource baselines.

This shall add no global context/root state, overload/environment table,
dispatcher, function pointer, indirect call, lookup, replay, allocation,
foreign dependency, C/C++ runtime, other-language standard library, public ABI,
or `topal-native/6` revision. Future library metadata shall retain canonical
source-session and context/root identity, call position, source-ordered overload
set and exact selected declaration, selection evidence/conversions, recursion
proof identity, member stable identity/declaration position, complete semantic
classifier/representation, ordered capture schema/lifetime/effects, and a
versioned target adapter independently of private capture names, LLVM
types/symbols, debug shadows, and physical placement. This realizes
`TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001` for increments 6b2b2f and 6c2d.

## TOPAL-COMP-LOCAL-FUNCTION-ENVIRONMENT-001 — Exact local named Function environments

Within an ordinary root-function invocation, the checked model shall retain an
already-visible named root Function through source-ordered local alias bindings
and shall follow an exact directly applied alias back to its original visible
declaration snapshot while discovering context/root captures. The same scan
shall follow an already-admitted non-escaping ordinary nested Function called
directly or through an exact local alias. Shadowing or rebinding to any other
value shall terminate the retained edge.

An alias shall preserve original function identity, staticness, source-ordered
visible overload declarations, and selection behavior. Overloaded alias calls
shall reuse only the evidence admitted by
`TOPAL-COMP-OVERLOAD-ENVIRONMENT-001` and shall select one exact capture vector.
A nested Function shall retain existing lexical captures, but the model shall
exclude standard `@ member` and `root member` parameters from that lexical
vector and add them exactly once through the environment group. Alias discovery
shall neither prove recursion nor admit unsupported nested declarations,
overload choices, capture representations, or escapes.

The Linux x86-64 backend shall emit matching direct private `fastcc` definitions
and calls for outer, nested, and selected target functions, leaving physical
placement to LLVM and requiring no optimization. Full O0 DWARF/GDB shall expose
each Function alias, selected frame, explicit parameter, and exact source-named
capture in active and suspended frames. Tests shall cover scalar and aggregate
context/root values, multi-binding alias chains, overloaded alias selection,
nested direct and aliased calls, lexical shadow isolation, value-fact-dependent
artifact-free rejection, checked capture vectors, exact output/IR, interpreter
modes, reversible history, freestanding ELF/DWARF, GDB frames, the shared
corpus, and separate resource baselines.

This shall add no closure/environment object, Function dispatcher, function
pointer, indirect call, global context/root state, lookup, replay, allocation,
foreign dependency, C/C++ runtime, other-language standard library, public ABI,
or `topal-native/6` revision. Exact private Function boundaries carrying these
environments are admitted by
`TOPAL-COMP-FUNCTION-ENVIRONMENT-BOUNDARY-001`. Escaping nested Functions;
recursive/overloaded nested Functions; opaque or dynamically selected aliases;
and public/library local-Function environments remain deferred. Future library metadata shall retain canonical source-session
and context/root identities, lexical invocation scope, alias binding stable
identity/declaration position, retained root Function and visible overload
snapshot or nested declaration path, selected declaration/call edge, selection
evidence/conversions, recursion proof identity, ordered capture
schema/classifiers/representations/lifetimes/effects, and a versioned target
adapter independently of observation tags, private capture names, LLVM
types/symbols, debug shadows, and physical placement. This realizes
`TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001` for increments 6b2b2g and 6c2e.

## TOPAL-COMP-FUNCTION-ENVIRONMENT-BOUNDARY-001 — Exact private Function-environment boundaries

The checked compiler model shall preserve the complete exact environment of a
named root Function value and retained overload set, an anonymous Function
constructed within an ordinary root-function invocation, or an admitted nested
Function across private scalar Function parameters and
results and represented Tuple or labeled Record parameters and results that
contain Function leaves. Each Function leaf shall retain one exact callable
identity and complete semantic capture schema; opaque and dynamically selected
values shall remain `E-COMPILER-UNSUPPORTED`.

The environment transported for a named root overload set shall be the
deduplicated union required by its retained declarations in root declaration
order, with context members before root members. Declaration identities,
selection evidence, and per-declaration capture schemas shall remain separate
from the runtime Function tag. After admitted source-ordered selection, the
backend shall pass only the selected declaration's vector to the direct target.
Value-fact-dependent higher-order selection shall fail before artifact
publication. Anonymous and nested captures shall retain each lexical,
contextual, and root value once; multiple boundaries shall forward the original
immutable values without initializer replay or caller-frame lookup.

The Linux x86-64 backend shall represent private Function results together with
their ordered captures and shall associate Function-containing aggregate result
captures with canonical Tuple-index or Record-label paths. It shall emit
matching private `fastcc` definitions and direct calls for every specialized
boundary and selected target, leaving aggregate classification and physical
placement to LLVM and requiring no optimization. Full O0 DWARF/GDB shall expose
source Function values, exact source-named captures, and selected, anonymous,
and nested frames. Tests shall cover named overload sets, scalar and Record
parameters/results, anonymous and nested values, context/root scalar and
aggregate captures, exact checked vectors, output/IR, value-fact-dependent
artifact-free rejection, interpreter modes, reversible history, freestanding
ELF/DWARF, GDB frames, the shared corpus, and separate resource baselines.

This shall add no global context/root state, closure/environment object,
overload/environment table, runtime dispatcher, function pointer, indirect
call, lookup, replay, allocation, foreign dependency, C/C++ runtime,
other-language standard library, public ABI, or `topal-native/6` revision. Sum
or unsupported Function-containing representations; escaping anonymous results
or nested results outside `TOPAL-COMP-NESTED-FUNCTION-ESCAPE-001`;
recursive/overloaded nested Functions; fact-dependent, opaque, or
dynamic selection; and public/library Function environments remain deferred.
Future library metadata shall retain canonical source-session, context/root,
lexical-scope, callable, declaration, overload-snapshot, aggregate-path, and
member identities; selection evidence/conversions; recursion-proof identity;
semantic classifiers/representations; ordered per-declaration and transported
capture schemas/lifetimes/effects; and a versioned target adapter independently
of runtime tags, compiler-private names, LLVM types/symbols, debug shadows, and
physical placement. This realizes
`TOPAL-COMPILER-FUNCTION-ENVIRONMENT-BOUNDARY-001` for increments 6b2b2h and
6c2f.

## TOPAL-COMP-RECURSION-INT-001 — Proven direct decreasing Int recursion

The checked compiler model shall reuse the shared structural proof for
`TOPAL-FUNCTION-RECURSION-INT-001` and
`TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001`. It shall reserve one function
symbol only after that proof succeeds, direct every self-call in the recursive
action to that symbol, and support every independently proven self-call in the
action. A self-call in the base, a zero or otherwise invalid literal step, and
an indirect cycle without an implemented complete-cycle proof shall remain
`E-COMPILER-UNSUPPORTED`.

The recursive function body shall discard initial-call range, Rational, and
String facts for its parameters and check each against only its declared
classifier. LLVM definitions and calls shall use the same exact private
`fastcc` prototype and shall retain `noinline` without claiming `norecurse` or
requiring a tail-call optimization. Recursive Int values shall use the existing
immutable exact representation and Topal-owned Linux allocation/syscall layer,
with no runtime dispatch, C/C++ runtime, other-language standard library, or
`topal-native/6` revision. DWARF/GDB shall retain distinct recursive frames and
the current Int parameter. This requirement realizes
`TOPAL-COMPILER-RECURSION-INT-001` for increment 3b2-b5e1.

## TOPAL-COMP-RECURSION-INT-INCREASING-001 — Proven direct increasing Int recursion

The checked compiler model shall also admit the shared
`TOPAL-FUNCTION-RECURSION-INT-INCREASING-001` proof for a unary `Int` overload
with an inclusive upper-bound base and strict positive literal additions. It
shall accept multiple-unit progress in either proven direction, including safe
overshoot of the inclusive bound, while zero, negative, runtime-selected,
subtracting, base-action, and otherwise unproven increasing edges remain
`E-COMPILER-UNSUPPORTED`.

Increasing recursion shall use the same generalized parameter facts, reserved
overload symbol, exact private `fastcc` prototype, `noinline` and non-`norecurse`
attributes, immutable exact Int representation, and Topal-owned Linux
allocation/syscall layer as decreasing recursion. GDB shall retain consecutive
frames and their increasing parameter values. This adds no runtime dispatch,
C/C++ runtime, other-language standard library, or `topal-native/6` revision
and realizes `TOPAL-COMPILER-RECURSION-INT-INCREASING-001` for increment
3b2-b5e2.

## TOPAL-COMP-RECURSION-NAT-001 — Proven direct Nat recursion

The checked compiler model shall admit the shared
`TOPAL-FUNCTION-RECURSION-NAT-001` and
`TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001` proofs. Decreasing recursion shall
preserve nonnegativity by enforcing the proof's nonnegative inclusive bound and
maximum literal step; increasing recursion shall add a strict positive literal.
Unsafe overshoot and every otherwise unproven direct edge shall remain rejected.

Within a recursive step the model shall forget `Nat` evidence to the unchanged
exact Int carrier for addition or subtraction, then attach `Nat` evidence to
the recursive argument only under the shared range-preservation proof. That
proof-backed `IntToNat` node shall lower as an identity: it shall not call the
dynamic Nat validator, allocate, reinterpret the value as unsigned, or change
its representation. The recursive function shall retain an exact private
`fastcc` prototype, `noinline` without `norecurse`, O0 correctness, distinct
frames, and `Nat` parameter identity in DWARF/GDB. It shall use only the
existing Topal-owned Linux syscall runtime and `topal-native/6` ABI, with no
C/C++ runtime or other-language standard library. This realizes
`TOPAL-COMPILER-RECURSION-NAT-001` for increment 3b2-b5e3.

## TOPAL-COMP-DECREASES-001 — Explicit measured recursion

The checked compiler model shall retain a v0.1 `Decreases` effect bound and
admit its direct recursive edge only when the shared
`TOPAL-FUNCTION-DECREASES-001` proof verifies the complete overload body. The
initial closure shall support one directly named `Int` or `Nat` measure among
multiple scalar parameters, verify the corresponding packaged argument on each
self-call, and reject mismatched measures and non-progressing steps.

Active proof metadata shall record the exact measured parameter index. A
proof-backed Nat evidence conversion shall be available only at that position;
all unmeasured parameters shall retain ordinary checking. The measure shall be
erased before lowering and add no hidden argument, counter, allocation,
validation call, or ABI field. The function shall use one exact private
`fastcc` prototype with `noinline` and no false `norecurse`, preserve every
source parameter in recursive DWARF/GDB frames, and use only the existing
Topal-owned Linux syscall runtime. No C/C++ runtime, other-language standard
library, or `topal-native/6` revision is permitted. This realizes
`TOPAL-COMPILER-FUNCTION-DECREASES-001` for increment 3b2-b5e4.

## TOPAL-COMP-RECURSION-OVERLOAD-IDENTITY-001 — Overload-specific recursion identity

The checked compiler model shall use function name, staticness, and the complete
selected input classifier sequence as an active call-graph identity. A call
from one overload to a same-named overload with a different input header shall
be an ordinary acyclic edge and shall not consume or require the active
overload's recursion proof. A later edge returning to an active complete
identity shall remain subject to the applicable proof gate.

Every selected overload shall retain a distinct compiler-private symbol,
checked source signature, and DWARF subprogram even when its scalar LLVM
prototype uses the same carrier types. The acyclic callee shall precede its
caller in generated definitions and appear as a distinct nested GDB frame. No
runtime dispatch, type tag, foreign dependency, C/C++ runtime, other-language
standard library, or `topal-native/6` revision is permitted. This realizes
`TOPAL-COMPILER-RECURSION-OVERLOAD-IDENTITY-001` for increment 3b2-b5e5.

## TOPAL-COMP-RECURSION-INT-MUTUAL-001 — Proven mutual Int recursion

The checked compiler model shall reuse the shared decreasing and increasing
mutual `Int` edge proofs. Before admitting a return to an active overload, it
shall verify an active slice of at least two members in which every proof rule
is identical, every next-member name matches the following active identity,
and the last member targets the first. Isolated candidates, mixed directions,
zero or otherwise invalid steps, and incomplete cycles shall remain
`E-COMPILER-UNSUPPORTED`. Multiple next-member calls shall be checked and
instantiated independently.

Every member shall use its reserved exact private `fastcc` prototype. Generated
definitions shall retain `noinline`, omit the false `norecurse` attribute, and
require no tail-call optimization for O0 correctness. Distinct source members,
parameters, and nested frames shall remain available through DWARF/GDB. The
cycle proof shall be erased before lowering and add no dispatch table, hidden
state, C/C++ runtime, other-language standard library, or `topal-native/6`
revision. This realizes `TOPAL-COMPILER-RECURSION-INT-MUTUAL-001` for increment
3b2-b5e6.

## TOPAL-COMP-RECURSION-NAT-MUTUAL-001 — Proven mutual Nat recursion

The checked compiler model shall reuse the shared decreasing and increasing
mutual `Nat` edge proofs and the complete active-cycle check. All active members
shall have the same direction-specific rule, each next-member name shall match
the following selected overload, and the final target shall close a cycle of at
least two members. Every decreasing edge shall satisfy its own nonnegative bound
and literal-step limit. Unsafe overshoot, mixed directions, invalid steps, and
incomplete cycles shall remain `E-COMPILER-UNSUPPORTED`.

The current active member may restore `Nat` evidence only on the unary argument
to the next member named by its proof. The checked `IntToNat` boundary shall emit
no call to `topal.runtime.int.try.to.nat`; unrelated arithmetic receives no such
authority. Every member shall use its reserved exact private `fastcc` prototype,
retain `noinline`, omit `norecurse`, and expose distinct Nat parameters and
recursive frames through DWARF/GDB. No unsigned carrier, dispatcher, hidden
state, C/C++ runtime, other-language standard library, or `topal-native/6`
revision is permitted. This realizes
`TOPAL-COMPILER-RECURSION-NAT-MUTUAL-001` for increment 3b2-b5e7.

## TOPAL-COMP-RETURN-001 — Direct explicit function return

Within an admitted linear function body, `return expression` shall evaluate and
validate the expression once, preserve every preceding statement in order, and
complete the current invocation without emitting the unreachable tail. It shall
use the same private result representation and DWARF source mapping as an
implicit final result. A return at root shall remain a source diagnostic.

This requirement covers the direct-body case of
`TOPAL-FUNCTION-RETURN-001` and realizes `TOPAL-COMPILER-RETURN-001` for
compiler increment 3b2-a. Returns from nested lexical blocks remain with their
scope and cleanup lowering in increment 3b2-b2.

## TOPAL-COMP-BLOCK-001 — Lexical block values

The compiler shall evaluate an admitted lexical block in a fresh checked and
generated-value environment nested inside its enclosing environment. An empty
block shall produce Unit without allocation. A nonempty block shall execute
statements in source order and deliver its final value; its bindings may shadow
outer names and shall not escape. An inner initializer shall resolve names
before introducing its own binding.

Generated instructions and machine-represented immutable locals shall use a
nested DWARF lexical scope so GDB resolves the innermost visible binding. The
compiler shall reject nested declarations and return-through-block until their
declaration, cleanup, and exit-edge lowerings are admitted.

This requirement covers the cleanup-free block subset of
`TOPAL-EXEC-BLOCK-001` and the block case of `TOPAL-SYN-GRAMMAR-001`. It
realizes `TOPAL-COMPILER-BLOCK-001` for compiler increment 3b2-b1.

## TOPAL-COMP-COMPLETED-001 — Completion evidence

The compiler shall keep Completed distinct from Unit in the checked model,
generated signatures, values, equality, display, and debug information. An
admitted function returning Completed shall have a retained typed result and
call boundary at O0 so a dependent continuation remains ordered after the
call's completion; it shall not be lowered as a Unit-returning `void` call.

The zero-data value shall require no allocation or foreign runtime. Its private
machine carrier shall remain sealed inside the target-qualified Topal ABI, and
DWARF/GDB shall render the source identity `Completed` rather than an unrelated
integer or Unit value.

This requirement covers `TOPAL-EXEC-COMPLETED-001` and realizes
`TOPAL-COMPILER-COMPLETED-001` for compiler increment 3b2-b2.

## TOPAL-COMP-EFFECT-EMPTY-001 — Canonical empty Effect value

The checked compiler model shall classify `Effects ()` as the canonical empty
`Effect` value without scheduling or performing an interaction. `Effect` shall
remain distinct from Unit and `Completed` through classified bindings,
same-classifier equality, decomposed positional products, scalar function
parameters and results, and source display. This scalar rule does not itself
admit Effect Lists or aggregate results; `TOPAL-COMP-TUPLE-RESULT-001`
separately admits a qualified Tuple result containing this value, and
`TOPAL-COMP-LIST-EFFECT-001` admits the first immutable List representation.

The backend shall use a sealed zero-data scalar only in Topal-private signatures,
emit a distinct `Effect` DWARF enumeration for GDB, and require neither
allocation nor an effect-specific runtime function. Canonical display shall use
the existing Topal-owned Linux write boundary. No C/C++ runtime, other-language
standard library, public integer ABI, or `topal-native/6` revision is permitted.
This realizes `TOPAL-COMPILER-EFFECT-EMPTY-001` for compiler increment 7a.

## TOPAL-COMP-FUNCTION-EMPTY-EFFECT-001 — Explicit empty function effect bound

The checked compiler model shall admit the v0.1 post-result
`: Effects ()` upper bound on an otherwise admitted ordinary function. Because
the native subset rejects every source operation with a nonempty inferred
effect, it shall prove the implementation row exactly empty, check containment,
and retain a canonical explicit-empty row on the checked declaration and every
selected function instance. Unsupported nonempty, alternative, or polymorphic
rows shall receive a stable checked diagnostic instead of being erased or
treated as empty. Existing `Decreases` proof annotations shall retain their
separate behavior.

For one exact visible overload carrying the explicit empty bound,
`lang view function` shall produce a static-only typed Function view retaining
root identity, input and result classifiers, staticness, and the declared row.
The view may initialize a binding or be discarded, but it shall not enter the
runtime binding environment. Runtime observation, aggregate or function
passage, root data publication, zero- or multi-overload views, and other
introspection shall remain `E-COMPILER-UNSUPPORTED`.

Code generation shall erase the view and lower the called function exactly as
an inferred-empty direct private function. No view local or effect descriptor
shall enter LLVM IR, DWARF, or the executable; the ordinary function frame and
parameters shall remain fully debuggable. This shall add no reflection runtime,
descriptor registry, dispatch, hidden effect argument, allocation, foreign
runtime, C/C++ standard library, undefined symbol, needed library, relocation,
public/serialized/library effect ABI, or `topal-native/6` revision. It realizes
`TOPAL-COMPILER-FUNCTION-EMPTY-EFFECT-001`,
`TOPAL-FUNCTION-EFFECT-BOUND-001`, `TOPAL-EFFECT-CONTAIN-001`,
`TOPAL-INTRO-STATIC-001`, and `TOPAL-INTRO-VIEW-001` for compiler increment
7a1.

## TOPAL-COMP-LIST-EFFECT-001 — Immutable Effect List foundation

The checked compiler model shall admit `List Effect` where a classifier gives
the contextual element type, construct `Empty` and `Entry (value, remaining)`
with exact homogeneous typing, and retain the List classifier through immutable
bindings and ordinary function parameters and results. Construction shall
evaluate the entry value and remaining List in source order. Canonical output
shall match the shared interpreter's recursive `Entry`/`Empty` spelling.

The Linux x86-64 backend shall represent `Empty` as a null private pointer and
each `Entry` as an immutable, naturally aligned 16-byte node containing the
zero-data Effect carrier and the remaining-node pointer. It shall allocate
nodes only through the existing Topal-owned Linux mapping boundary, retain them
safely for process lifetime, and express private function passage as an LLVM
pointer so LLVM owns physical AMD64 calling-convention lowering. DWARF shall
expose the semantic `List Effect` identity and node shape, and the bundled GDB
renderer shall validate, bound, and render finite chains without recursive
host-stack growth.

This first container slice shall add no List equality, decisions, traversal,
mutation, reclamation contract, other element type, public or serialized node
layout, C/C++ runtime, other-language standard library, foreign allocator, or
`topal-native/6` revision. It realizes `TOPAL-COMPILER-LIST-EFFECT-001` and
`TOPAL-TYPE-LIST-CONSTRUCT-001` for compiler increment 4b3d-a.

## TOPAL-COMP-LIST-BOOLEAN-001 — Ordinary immutable Boolean Lists

The checked compiler model shall admit contextual `Empty` and `Entry`
construction for `List Boolean`, immutable bindings, ordinary private
parameters and results, package fields, recursively admitted Tuple/Record
fields, structural equality and inequality, complete `Empty`/`Entry` decisions,
entry count, emptiness, canonical display, and debugging. Every constructor and
call operand shall evaluate once in source order. Unsupported Boolean-List
transforms shall remain `E-COMPILER-UNSUPPORTED` before artifact publication.

The Linux x86-64 backend shall represent `Empty` as a null private pointer and
each `Entry` as an immutable naturally aligned 16-byte node with its i1 payload
at offset zero and remaining-node pointer at offset eight. Padding shall carry
no source value. A compiler-selected runtime fragment shall implement finite
nonrecursive structural equality and entry counting; emptiness shall remain a
null test and decisions shall load the exact i1 payload. Correctness at O0 shall
not depend on optimization. Private pointer-bearing definitions, calls,
returns, Tuple/Record/package fields, target-derived DWARF, and the bounded GDB
renderer shall preserve the complete source classifier while LLVM owns physical
AMD64 placement.

Tests shall cover empty and nonempty construction, true and false entries,
same/different structural equality, complete decomposition, count, emptiness,
private parameter/result, package, Tuple, and Record passage, display, all
interpreter modes, reversible history, exact private IR, artifact-free
unsupported-transform rejection, freestanding ELF/DWARF, GDB values/frames,
the shared corpus, and separate interpreter/compiler resource baselines.

This increment shall add no runtime type tag, type-erased generic List, public,
foreign, serialized, or compiled-library node ABI, foreign allocator, C/C++
runtime, other-language standard library, or `topal-native/6` revision.
Projections, insertion, concatenation, reversal, removal, range selection,
traversal, higher-order transforms, final reclamation, and other element types
remain separately governed. Future library metadata shall retain element
classification, representation/ownership/lifetime/effects, and versioned
target-adapter facts independently of node offsets, private helper names, LLVM
types/symbols, debug shadows, and physical placement. This realizes
`TOPAL-COMPILER-LIST-BOOLEAN-001`, `TOPAL-TYPE-LIST-CONSTRUCT-001`,
`TOPAL-DECISION-LIST-001`, `TOPAL-TYPE-LIST-EQUALITY-001`,
`TOPAL-LIST-ENTRY-COUNT-001`, and `TOPAL-LIST-EMPTY-PREDICATE-001` for compiler
increment 4b3d-m.

## TOPAL-COMP-LIST-STRING-CORE-001 — Ordinary immutable String Lists

The checked compiler model shall admit contextual `Empty` and `Entry`
construction for `List String`, immutable bindings, ordinary private parameters
and results, package fields, recursively admitted Tuple/Record fields,
structural equality and inequality, complete `Empty`/`Entry` decisions, entry
count, emptiness, canonical display, and debugging. Every constructor and call
operand shall evaluate once in source order. Unsupported String-List transforms
shall remain `E-COMPILER-UNSUPPORTED` before artifact publication.

The Linux x86-64 backend shall represent `Empty` as a null private pointer and
each `Entry` as an immutable 16-byte node containing the existing String
descriptor pointer and remaining-node pointer. A compiler-selected runtime
fragment shall implement finite nonrecursive structural equality through the
canonical preserved-sequence String comparator and entry counting; emptiness
shall remain a null test and decisions shall load the exact descriptor pointer.
Correctness at O0 shall not depend on optimization. Private pointer-bearing
definitions, calls, returns, Tuple/Record/package fields, target-derived DWARF,
and the bounded GDB renderer shall preserve the complete source classifier and
String delimiters while LLVM owns physical AMD64 placement.

Tests shall cover empty and nonempty construction, distinct Unicode-capable
String entries, same/different structural equality, complete decomposition,
count, emptiness, private parameter/result, package, Tuple, and Record passage,
display, all interpreter modes, reversible history, exact private IR,
artifact-free unsupported-transform rejection, freestanding ELF/DWARF, GDB
values/frames, the shared corpus, and separate interpreter/compiler resource
baselines.

This increment shall add no copied String representation, host text API,
runtime type tag, type-erased generic List, public, foreign, serialized, or
compiled-library node ABI, foreign allocator, C/C++ runtime, other-language
standard library, or `topal-native/6` revision. Projections, insertion,
concatenation, reversal, removal, range selection, traversal, higher-order
transforms, final reclamation, and other element types remain separately
governed. Future library metadata shall retain element classification, String
and node representation/ownership/lifetime/effects, and versioned target-
adapter facts independently of node offsets, private helper names, LLVM
types/symbols, debug shadows, and physical placement. This realizes
`TOPAL-COMPILER-LIST-STRING-CORE-001`, `TOPAL-TYPE-LIST-CONSTRUCT-001`,
`TOPAL-DECISION-LIST-001`, `TOPAL-TYPE-LIST-EQUALITY-001`,
`TOPAL-LIST-ENTRY-COUNT-001`, and `TOPAL-LIST-EMPTY-PREDICATE-001` for compiler
increment 4b3d-n.

## TOPAL-COMP-LIST-CHARACTER-CORE-001 — Ordinary immutable Character Lists

The checked compiler model shall admit contextual `Empty` and `Entry`
construction for `List Character`, immutable bindings, ordinary private
parameters and results, package fields, recursively admitted Tuple/Record
fields, structural equality and inequality, complete `Empty`/`Entry` decisions,
entry count, emptiness, canonical display, and debugging. Every constructor and
call operand shall evaluate once in source order. Stored entries shall retain
their Character constraint evidence and complete preserved scalar sequence.
Unsupported Character-List transforms shall remain `E-COMPILER-UNSUPPORTED`
before artifact publication.

The Linux x86-64 backend shall represent `Empty` as a null private pointer and
each `Entry` as an immutable 16-byte node containing the constrained String
descriptor pointer and remaining-node pointer. The compiler-selected finite
String-List fragment shall implement structural equality through the canonical
preserved-sequence comparator and entry counting; emptiness shall remain a null
test and decisions shall load the exact descriptor pointer. Correctness at O0
shall not depend on optimization. Private pointer-bearing definitions, calls,
returns, Tuple/Record/package fields, target-derived DWARF, and the bounded GDB
renderer shall preserve the complete source classifier, multi-scalar character
contents, and delimiters while LLVM owns physical AMD64 placement.

Tests shall cover empty and nonempty construction, a multi-scalar grapheme and
a ZWJ emoji, same/different structural equality, complete decomposition, count,
emptiness, private parameter/result, package, Tuple, and Record passage,
display, all interpreter modes, reversible history, exact private IR,
artifact-free unsupported-transform rejection, freestanding ELF/DWARF, GDB
values/frames, the shared corpus, and separate interpreter/compiler resource
baselines.

This increment shall add no code-point Character representation, copied String,
implicit normalization, host text API, runtime type tag, type-erased generic
List, public, foreign, serialized, or compiled-library node ABI, foreign
allocator, C/C++ runtime, other-language standard library, or `topal-native/6`
revision. Projections, insertion, concatenation, reversal, removal, range
selection, traversal, higher-order transforms, final reclamation, and other
element types remain separately governed. Future library metadata shall retain
the Character classifier and constraint evidence, representation/ownership,
lifetime, effects, and versioned target-adapter facts independently of node
offsets, private helper names, LLVM types/symbols, debug shadows, and physical
placement. This realizes `TOPAL-COMPILER-LIST-CHARACTER-CORE-001`,
`TOPAL-TYPE-LIST-CONSTRUCT-001`, `TOPAL-DECISION-LIST-001`,
`TOPAL-TYPE-LIST-EQUALITY-001`, `TOPAL-LIST-ENTRY-COUNT-001`, and
`TOPAL-LIST-EMPTY-PREDICATE-001` for compiler increment 4b3d-q.

## TOPAL-COMP-LIST-NAT-CORE-001 — Ordinary immutable Nat Lists

The checked compiler model shall admit contextual `Empty` and `Entry`
construction for `List Nat`, immutable bindings, ordinary private parameters
and results, package fields, recursively admitted Tuple/Record fields,
structural equality and inequality, complete `Empty`/`Entry` decisions, entry
count, emptiness, canonical display, and debugging. Every constructor and call
operand shall evaluate once in source order. Finite entries shall be validated
as nonnegative, `+Infinity` shall remain admitted, and every entry shall retain
Nat evidence. Unsupported Nat-List transforms shall remain
`E-COMPILER-UNSUPPORTED` before artifact publication.

The Linux x86-64 backend shall represent `Empty` as a null private pointer and
each `Entry` as an immutable 16-byte node containing the exact Int-compatible
pointer and remaining-node pointer. The compiler-selected finite Int-List
fragment shall implement structural equality, including positive-infinity
comparison, and entry counting; emptiness shall remain a null test and decisions
shall load the exact numeric pointer. Correctness at O0 shall not depend on
optimization. Private pointer-bearing definitions, calls, returns,
Tuple/Record/package fields, target-derived DWARF, and the bounded GDB renderer
shall preserve the complete source classifier, arbitrary-precision entries,
and `+Infinity` while LLVM owns physical AMD64 placement.

Tests shall cover empty and nonempty construction, zero, a large finite Nat,
`+Infinity`, same/different structural equality, complete decomposition, count,
emptiness, private parameter/result, package, Tuple, and Record passage,
display, all interpreter modes, reversible history, exact private IR,
artifact-free unsupported-transform rejection, freestanding ELF/DWARF, GDB
values/frames, the shared corpus, and separate interpreter/compiler resource
baselines.

This increment shall add no machine-unsigned width, truncation, wrapping,
second numeric representation, runtime type tag, type-erased generic List,
public, foreign, serialized, or compiled-library node ABI, foreign allocator,
C/C++ runtime, other-language standard library, or `topal-native/6` revision.
Projections, insertion, concatenation, reversal, removal, range selection,
traversal, higher-order transforms, final reclamation, and other element types
remain separately governed. Future library metadata shall retain Nat constraint
evidence and infinity capability, representation/ownership, lifetime, effects,
and versioned target-adapter facts independently of node offsets, private helper
names, LLVM types/symbols, debug shadows, and physical placement. This realizes
`TOPAL-COMPILER-LIST-NAT-CORE-001`, `TOPAL-NUM-NAT-001`,
`TOPAL-TYPE-LIST-CONSTRUCT-001`, `TOPAL-DECISION-LIST-001`,
`TOPAL-TYPE-LIST-EQUALITY-001`, `TOPAL-LIST-ENTRY-COUNT-001`, and
`TOPAL-LIST-EMPTY-PREDICATE-001` for compiler increment 4b3d-r.

## TOPAL-COMP-LIST-INT-CONTAINMENT-001 — Exact Int List containment

The checked compiler model shall extend contextual homogeneous List
construction and private function passage to `List Int` without changing Int
semantics. It shall evaluate the complete List operand before the entry or List
pattern operand exactly once. `contains-entry` shall search for any equal Int;
`contains-sequence` shall search for a consecutive equal pattern; and
`contains-subsequence` shall search for the ordered equal pattern while
permitting gaps. An empty sequence or subsequence pattern shall match every
List. Every input shall remain immutable.

The backend shall retain the private 16-byte node size while using an exact Int
pointer in the first word and the remaining-node pointer in the second word.
Containment shall use allocation-free loops and the existing canonical
arbitrary-precision Int comparator; correctness at O0 shall not depend on LLVM
optimization. The compiler shall include these specialized runtime helpers
only when a checked expression requires them. Canonical display, opaque-pointer
private calls and returns, DWARF, and the bounded GDB renderer shall preserve
every Int value and the `List Int` classifier. LLVM shall continue to own
physical AMD64 pointer placement.

This increment shall add no public, foreign, serialized, or generic List ABI,
no mutation or reclamation contract, no C/C++ runtime, other-language standard
library, foreign allocator, or `topal-native/6` revision. List equality,
decisions, other observations and transformations, and other element types
remain deferred. It realizes `TOPAL-COMPILER-LIST-INT-CONTAINMENT-001`,
`TOPAL-TYPE-LIST-CONSTRUCT-001`, `TOPAL-LIST-CONTAINS-ENTRY-001`,
`TOPAL-LIST-CONTAINS-SEQUENCE-001`, and
`TOPAL-LIST-CONTAINS-SUBSEQUENCE-001` for compiler increment 4b3d-b.

## TOPAL-COMP-LIST-INT-REMOVAL-001 — Immutable Int List removal

The checked compiler model shall accept `remove-first` and `remove-all` only
for the admitted `List Int` specialization and an exact Int operand. It shall
evaluate the complete List before the removal value exactly once, preserve the
List classifier, and leave every input immutable. `remove-first` shall remove
only the earliest equal entry and preserve an unmatched List unchanged;
`remove-all` shall remove every equal entry. Both shall retain the order and
exact arbitrary-precision values of all other entries.

The backend shall implement both operations as finite nonrecursive loops using
the existing canonical Int comparator. It may share the unchanged input and an
untouched suffix. Rebuilt prefixes or filtered results shall contain immutable
logical 16-byte Int-pointer/remaining-pointer nodes allocated only through the
Topal-owned Linux mapping boundary. The compiler shall keep the common layout,
containment, and removal LLVM fragments independently conditional so unrelated
programs do not pay their assembly cost. Behavior at O0 shall not depend on an
optimization pass.

Canonical output, private pointer passage, List-specific DWARF, and bounded GDB
rendering shall remain exact. This increment shall add no public, foreign,
serialized, persistent, or generic List ABI, no foreign allocator, C/C++
runtime, other-language standard library, undefined symbol, needed library,
relocation, reclamation contract, or `topal-native/6` revision. It realizes
`TOPAL-COMPILER-LIST-INT-REMOVAL-001`, `TOPAL-LIST-REMOVE-FIRST-001`, and
`TOPAL-LIST-REMOVE-ALL-001` for compiler increment 4b3d-c.

## TOPAL-COMP-LIST-INT-CORE-001 — Basic immutable Int List operations

The checked compiler model shall admit the existing basic List surface for
`List Int`: explicit `empty List Int`, `one value`, prepend, append, concat,
reverse, entry count, emptiness, structural equality, first, rest, uncons, and
a complete `Empty`/`Entry (first, rest)` decision. Operands shall be evaluated
exactly once in source order. The model shall preserve arbitrary-precision Int
entries, retain exact result classifiers, scope decision bindings only within
the selected Entry action, and reject unsupported element classifiers rather
than applying a type-erased implementation.

The Linux x86-64 backend shall reuse the private immutable 16-byte
Int-pointer/remaining-pointer node. Concatenation shall copy its left operand
into one Topal-owned contiguous allocation and may share the right operand;
append and reverse shall publish only newly initialized immutable nodes.
Counting, emptiness, projections, decisions, and equality shall use finite
nonrecursive control flow. Equality shall call the canonical exact Int
comparator. Behavior at O0 shall not depend on an optimization pass.

`first`, `rest`, and `uncons` shall retain the existing private Optional header
while preserving semantic payload classifiers `Int`, `List Int`, and
`(Int, List Int)`. A present empty tail shall remain distinct from absence.
DWARF and the bundled GDB renderer shall name, validate, and render
`Optional List Int` and `Optional (Int, List Int)` values. The compiler shall
include the core LLVM fragment only for checked expressions that require it.

This increment shall add no public, foreign, serialized, persistent, or
generic List/Optional/pair ABI, foreign allocator, C/C++ runtime,
other-language standard library, undefined symbol, needed library, relocation,
reclamation contract, or `topal-native/6` revision. It realizes
`TOPAL-COMPILER-LIST-INT-CORE-001`, `TOPAL-TYPE-LIST-CONSTRUCT-001`,
`TOPAL-DECISION-LIST-001`, `TOPAL-TYPE-LIST-EQUALITY-001`,
`TOPAL-LIST-PREPEND-001`, `TOPAL-LIST-APPEND-001`,
`TOPAL-LIST-CONCAT-001`, `TOPAL-LIST-ENTRY-COUNT-001`,
`TOPAL-LIST-EMPTY-PREDICATE-001`, `TOPAL-LIST-EMPTY-001`,
`TOPAL-LIST-ONE-001`, `TOPAL-LIST-UNCONS-001`, `TOPAL-LIST-FIRST-001`,
`TOPAL-LIST-REST-001`, and `TOPAL-LIST-REVERSE-001` for compiler increment
4b3d-d.

## TOPAL-COMP-LIST-INT-FUNCTIONS-001 — Contextual Int List functions

The checked compiler model shall accept contextual binding-pattern anonymous
functions for `List Int` map, select, and Int-state fold. It shall infer and
require exact signatures `Int -> Int`, `Int -> Boolean`, and
`(Int, Int) -> Int`, respectively. It shall evaluate the List once, evaluate a
fold initial value once after the List, retain admitted immutable lexical
captures, diagnose arity or result mismatches, and reject anonymous product
patterns and other element/state types.

Generated map and select loops shall invoke the checked body exactly once per
entry in source order. Map shall publish transformed entries in that order;
select shall retain exactly accepted original Int values in that order. Fold
shall pass prior state before current entry, return its initial Int for Empty,
and otherwise return the final exact Int. The source List shall remain
unchanged. Arbitrary-precision operations inside every body shall reuse the
canonical Int runtime.

The Linux x86-64 backend shall specialize each body directly into finite LLVM
control flow. Map and select may allocate and link fresh private 16-byte nodes
through the Topal-owned mapping boundary while those nodes remain inaccessible;
published nodes shall be immutable. Fold shall carry its exact Int pointer in
an LLVM phi. Correctness at O0 shall require no host recursion, Function object,
callback ABI, indirect call, or traversal dispatcher. Existing List/Int DWARF
and the bundled GDB renderer shall expose parameters and result bindings.

This increment shall add no public, foreign, serialized, persistent, or
generic collection/callable ABI, foreign allocator, C/C++ runtime,
other-language standard library, undefined symbol, needed library, relocation,
reclamation contract, or `topal-native/6` revision. Traversal control and
remaining collection algorithms stay deferred. It realizes
`TOPAL-COMPILER-LIST-INT-FUNCTIONS-001`, `TOPAL-COLLECTION-MAP-001`,
`TOPAL-COLLECTION-SELECT-001`, `TOPAL-COLLECTION-FOLD-001`, and
`TOPAL-FUNCTION-ANONYMOUS-001` for compiler increment 4b3d-e.

## TOPAL-COMP-LIST-INT-BOUND-FUNCTIONS-001 — Bound anonymous List functions

The checked compiler model shall allow a binding-pattern anonymous Function to
be bound immutably and later supplied to the admitted `List Int` map, select,
or Int-state fold. The Function binding shall retain its parameter pattern,
body, static context, and defining lexical-capture facts. At each collection
use the compiler shall infer and require `Int -> Int`, `Int -> Boolean`, or
`(Int, Int) -> Int` as appropriate, preserving the definition-time snapshot
rather than resolving shadowing use-site bindings.

The backend shall keep the bound Function's existing private identity tag for
source display and debugging, but specialize the retained body directly into
the finite loop introduced by `TOPAL-COMP-LIST-INT-FUNCTIONS-001`. Execution
shall not inspect the tag or create a Function object, callback ABI, function
pointer, indirect call, closure allocation, or traversal dispatcher. Once-only
operand evaluation, per-entry source order, exact Int behavior, fresh immutable
result publication, empty identities, and O0 correctness shall remain
unchanged.

DWARF and GDB shall expose each bound anonymous Function identity alongside
the resulting List and Int bindings. This increment shall add no public,
foreign, serialized, persistent, or generic callable/collection ABI, foreign
allocator, C/C++ runtime, other-language standard library, undefined symbol,
needed library, relocation, or `topal-native/6` revision. Named, symbolic,
product-pattern, escaping, and dynamically selected collection functions and
other element/result/state classifiers remain deferred. It realizes
`TOPAL-COMPILER-LIST-INT-BOUND-FUNCTIONS-001`,
`TOPAL-COLLECTION-MAP-001`, `TOPAL-COLLECTION-SELECT-001`,
`TOPAL-COLLECTION-FOLD-001`, and `TOPAL-FUNCTION-ANONYMOUS-001` for compiler
increment 4b3d-f.

## TOPAL-COMP-RANGE-SELECTION-001 — Range-selected Lists and Strings

The checked compiler model shall admit `List Int select Range Int` and
`List Int select-index Range Int`, evaluating the operands once in source
order. The backend shall use exact arbitrary-precision entry comparisons for
value selection and exact zero-based positions for index selection, traverse
the finite source once in order, preserve occurrences, and publish only fully
initialized fresh immutable nodes. Empty, disjoint, and inverted ranges shall
produce Empty and the source shall remain unchanged.

For a closed compiler-known String and finite closed compiler-known `Range Int`,
the checked model shall perform index selection with the shared pinned Unicode
user-perceived-Character segmentation and lower the selected semantic result as
an ordinary String. It shall not materialize selection/slice provenance or make
storage sharing observable. Dynamic String range selection shall retain a
stable unsupported diagnostic until the compiler has a freestanding Topal
Unicode segmentation runtime; it shall not depend on a host or foreign Unicode
library merely to broaden this increment.

The Linux x86-64 backend shall conditionally link the private List selection
fragment only when used. That fragment may call the existing Topal allocator,
exact Int conversion/comparison, and Range-membership helpers, but shall add no
foreign allocator, C/C++ runtime, other-language standard library, undefined
symbol, needed library, dynamic relocation, public/serialized/persistent/generic
collection ABI, stabilized private layout, or `topal-native/6` revision. DWARF
and GDB shall retain the applicable List, Int, Range, String, binding, parameter,
and frame views. This realizes `TOPAL-COMPILER-RANGE-SELECTION-001`,
`TOPAL-RANGE-VALUE-SELECTION-001`, and `TOPAL-RANGE-INDEX-SELECTION-001` for
compiler increment 4b3d-g.

## TOPAL-COMP-TRAVERSAL-CONTROL-001 — Short-circuiting Int List fold

The checked compiler model shall admit `Continue Int` and `Finish Int` as
distinct `TraversalControl Int` values. An admitted `List Int` fold with an
Int initial state and an anonymous action may return either an ordinary Int
state or `TraversalControl Int`. Continue shall supply the exact state for the
next reached entry; Finish shall return its exact payload immediately without
executing the action for later entries. The List and initial state shall each
be evaluated once in source order, Empty shall return the initial state, and
ordinary Int-result folds shall remain unchanged. Other payload, element, and
state classifiers and traversal-control function boundaries shall retain a
stable unsupported diagnostic.

The Linux x86-64 backend shall lower each constructor to a private immutable
16-byte allocation containing a 64-bit Continue/Finish tag followed by the
canonical Int pointer. The specialized generated fold loop shall load the tag
and payload and branch directly to its advance or done block. Behavior at O0
shall require no optimization, callback ABI, indirect call, host recursion,
runtime traversal dispatcher, or separately linked traversal helper. DWARF and
the bundled GDB renderer shall identify `TraversalControl Int`, validate its
private storage, and render both constructors.

This increment shall use only the existing Topal-owned Linux mapping and write
boundaries and shall add no foreign allocator, C/C++ runtime, other-language
standard library, undefined symbol, needed library, dynamic relocation,
public/foreign/serialized/persistent/generic layout, or `topal-native/6`
revision. It realizes `TOPAL-COMPILER-TRAVERSAL-CONTROL-001`,
`TOPAL-EXEC-TRAVERSAL-CONTROL-001`, and `TOPAL-COLLECTION-FOLD-001` for compiler
increment 4b3d-h.

## TOPAL-COMP-LIST-INT-PAIR-MAP-001 — Int-pair List product map

The checked compiler model shall admit expected `List (Int, Int)` construction
and map it with a single two-field anonymous product parameter pattern whose
bindings are both exact Int. The anonymous action may be directly contextual
or retained in an immutable binding. Each invocation shall bind fields in
source order and return one Int result. Mapping shall visit every pair exactly
once in List order, preserve every exact arbitrary-precision value, publish a
`List Int`, and leave the source unchanged. Duplicate bindings, wrong product
arity, non-Int fields/results, select/fold use, and unsupported pair-List
boundaries shall receive stable checked diagnostics.

The Linux x86-64 backend shall store each pair inline as two canonical Int
pointers followed by the remaining-node pointer in one naturally aligned
24-byte private node. Construction shall evaluate the pair and remaining List
before allocating and fully initializing the node. The generated map loop
shall load both fields directly into the specialized anonymous environment and
reuse the existing immutable `List Int` result loop. Canonical output, semantic
DWARF, and the bounded validating GDB renderer shall preserve complete pair
Lists and mapped results at O0.

This increment shall use only the existing Topal Linux mapping and write
boundaries. It shall add no tuple payload allocation, generic/type-erased List
runtime, callback convention, indirect call, foreign allocator, C/C++ runtime,
other-language standard library, undefined symbol, needed library, dynamic
relocation, public/foreign/serialized/persistent/generic List ABI, or
`topal-native/6` revision. It realizes
`TOPAL-COMPILER-LIST-INT-PAIR-MAP-001`, `TOPAL-TYPE-LIST-CONSTRUCT-001`,
`TOPAL-COLLECTION-MAP-001`, and `TOPAL-FUNCTION-ANONYMOUS-001` for compiler
increment 4b3d-i.

## TOPAL-COMP-LIST-RECURSIVE-001 — Exact recursive Int/String Lists

The checked compiler model shall admit contextual `List (Int, String)` and
`List List (Int, String)` construction and retain both exact classifiers. The
outer recursive List shall cross ordinary parameter/result boundaries through
one direct specialization. Its admitted observations shall be `first` with
`Optional (List (Int, String))`, exact `entry-count`, and structural equality
that preserves outer and inner order and compares every exact Int and String.
Construction and observations shall evaluate each source operand once in
source order; `Some Empty` shall remain distinct from `None`. Direct inner pair-
List boundaries/equality, outer `rest`/`uncons`, deeper nesting, other shapes,
and all unlisted List operations shall receive stable checked diagnostics.

The Linux x86-64 backend shall use a private 24-byte inner node containing Int,
String, and remaining pointers, and a private 16-byte outer node containing
inner-List and remaining pointers. Empty shall be null at both levels. Nodes
shall be allocated only through the Topal-owned Linux mapping boundary and
shall be immutable after complete initialization. Independently selected,
shape-exact LLVM loops shall implement outer count, first, and equality; the
inner equality loop shall call the canonical exact Int and String comparators.
Behavior at O0 shall require no LLVM optimization, host recursion, generic or
type-erased List runtime, tag, callback, indirect call, or dispatcher.

The outer private function boundary shall use matching LLVM pointer prototypes
so LLVM owns physical AMD64 lowering. Target-layout-derived DWARF and the
bounded validating GDB renderer shall preserve and render both recursive List
types. The executable shall retain no undefined symbol, needed library, or
dynamic relocation and shall use no foreign allocator, C/C++ runtime, or other-
language standard library. This increment shall define no public, foreign,
serialized, persistent, generic, or compiled-library List ABI, stabilize no
private layout, and make no `topal-native/6` revision. It realizes
`TOPAL-COMPILER-LIST-RECURSIVE-001`, `TOPAL-TYPE-LIST-CONSTRUCT-001`,
`TOPAL-TYPE-LIST-RECURSIVE-001`, `TOPAL-TYPE-LIST-EQUALITY-001`,
`TOPAL-LIST-FIRST-001`, and `TOPAL-LIST-ENTRY-COUNT-001` for compiler increment
4b3d-j.

## TOPAL-COMP-LIST-SEQUENCE-001 — Closed ordered List sequence operations

The checked compiler shall admit the unchanged
`examples/language/list-sequence-operations.t` regression. Over exact finite
`List Int` values it shall implement single-value and List `insert-at`,
`split-at`, `take`, `drop`, indexed `remove`, range and anonymous-predicate
`remove-indexes`, anonymous-predicate `remove-values`, `zip-exact`,
`zip-shortest`, defaulted `zip-longest`, `unzip`, ordered `foreach`, `entries`,
and List identity collection. It shall also admit contextual `List String`
construction and ordered collection to String. Results, source preservation,
entry order, predicate order, and canonical output shall match the interpreter.

The checked model shall retain element, pair, indexed-entry, Result, and String
classifiers. A closed boundary, index, or index range shall be proven against
the retained exact source count before LLVM; an invalid one shall produce
`E-LIST-BOUNDARY-OUT-OF-RANGE`. Dynamic positions shall remain rejected until
the general evidence-dependent Result boundary is implemented. `zip-exact`
shall check counts in generated code and return the formal `out-of-range`
Result in domain `root.zip-exact(List,List)` when they differ. Predicates and
foreach actions shall bind each visited value or zero-based index exactly once,
in source order, at LLVM O0.

The Linux x86-64 backend shall use private immutable nodes selected from the
complete checked element shape: two pointers for Int and String entries, three
pointers for Int pairs, and the target-laid-out indexed-entry record plus its
remaining pointer. It shall use direct LLVM loops and private non-inlined
helpers for copying prefixes, sharing immutable suffixes, zipping, unzipping,
entry construction, and String concatenation. All allocation and output shall
continue through the Topal-owned Linux syscall runtime. LLVM target data layout
shall determine physical alignment; the backend shall hard-code no public
System V aggregate or call boundary. Correctness shall not depend on an LLVM
optimization pass.

DWARF and the bounded GDB renderer shall expose `List Int`, `List String`,
`List (Int, Int)`, `List (index : Int, value : Int)`, and the collected String.
The executable shall have no undefined symbol, needed library, dynamic
relocation, foreign allocator, C/C++ runtime, or other-language standard
library. This increment shall define no public, foreign, serialized,
persistent, generic, or compiled-library List ABI, stabilize no private node
layout, and make no `topal-native/6` revision. Future library metadata shall
encode the recursive List identity, element and product/record classifiers,
checked count evidence, operation and predicate identities, ordering,
fallibility, allocation effects, ownership, native-representation identity,
and target adapters rather than private node offsets or helper symbols. This
realizes `TOPAL-COMPILER-LIST-SEQUENCE-001`, `TOPAL-LIST-BOUNDARY-CHECK-001`
through `TOPAL-LIST-UNZIP-001`, `TOPAL-COLLECTION-FOREACH-001`,
`TOPAL-COLLECTION-ENTRIES-001`, `TOPAL-COLLECTION-COLLECT-LIST-001`, and
`TOPAL-COLLECTION-COLLECT-STRING-001` for compiler increment 5ay.

## TOPAL-COMP-FUNDAMENTAL-CONTAINERS-001 — Closed fundamental containers

The checked compiler shall admit the unchanged
`examples/language/fundamental-containers.t` regression. From an exact finite
`List Int` it shall construct the exact-extent Array, duplicate-eliminating Set,
and multiplicity-preserving Bag. From an exact finite
`List (String, Int)` it shall construct a Map under an explicit `reject`,
`keep-first`, or `keep-last` collision policy. It shall implement generic
`entry-count` and `empty?`, checked static Array indexing, Set membership, Bag
multiplicity, and Map lookup. Results and canonical output shall match the
interpreter without assigning an ordering guarantee to Set, Bag, or Map.

The checked model shall retain the container kind, element/key/value
classifiers, exact Array extent, and Map collision policy. A closed duplicate
key under `reject` shall produce `E-MAP-KEY-COLLISION` before LLVM. A closed
Array index shall be proven nonnegative and exact before lowering; a dynamic
index, dynamically discovered rejected collision, other classifier shape, or
general function boundary shall remain rejected until the necessary runtime
evidence and representation boundary is implemented.

The Linux x86-64 backend shall represent each container by a private immutable
pointer-backed header. Array may retain the source immutable List. Set, Bag,
and Map collectors may mutate only unpublished freshly allocated nodes while
eliminating duplicates, accumulating multiplicities, or resolving collisions;
they shall publish a completely initialized immutable result. Direct LLVM
helpers shall use the canonical exact Int and String operations and Topal-owned
allocation. Correctness at O0 shall not depend on an optimization pass, host
container, callback, indirect dispatch, foreign allocator, C/C++ runtime, or
another language's standard library.

Target-layout-derived DWARF and the bounded validating GDB renderer shall
expose semantic Array, Set, Bag, and Map types and complete values while keeping
node layout private. The executable shall have no undefined symbol, needed
library, or dynamic relocation. This increment shall define no public, foreign,
serialized, persistent, generic, or compiled-library container ABI, stabilize
no private offsets or helper symbols, and make no `topal-native/6` revision.
Future compiled-library metadata shall encode container kind, element/key/value
classifiers, Array extent, equality and collision policy, count/multiplicity/
lookup/index fallibility, ordering guarantees, allocation effects, ownership,
native-representation identity, and target adapters rather than private
layout. This realizes `TOPAL-COMPILER-FUNDAMENTAL-CONTAINERS-001`,
`TOPAL-ARRAY-COLLECT-001`, `TOPAL-SET-COLLECT-001`,
`TOPAL-BAG-COLLECT-001`, `TOPAL-MAP-COLLECT-001`,
`TOPAL-COLLECTION-ENTRY-COUNT-001`,
`TOPAL-COLLECTION-EMPTY-PREDICATE-001`,
`TOPAL-ARRAY-GET-CHECKED-001`, `TOPAL-MAP-LOOKUP-001`,
`TOPAL-SET-CONTAINS-001`, and `TOPAL-BAG-MULTIPLICITY-001` for compiler
increment 5az.

## TOPAL-COMP-TUPLE-RESULT-001 — Private positional-product results

The checked compiler model shall admit an ordinary or static Tuple result when
every recursively nested leaf has an exact supported private representation.
It shall retain the source field order and identities and reject unsupported
leaves rather than inventing a layout or conversion. Record fields admitted by
`TOPAL-COMP-RECORD-BOUNDARY-001` may occur recursively. Function execution and
all field evaluation shall remain correct
at O0 without semantic heap storage, runtime allocation, or optimization.

The Linux x86-64 backend shall lower each admitted result to a recursively
nested, non-packed LLVM literal struct. Definition and calls shall use the same
exact private `fastcc` prototype, with `insertvalue` and `extractvalue` forming
and observing the SSA value while LLVM performs target-specific physical call
lowering. The representation shall remain module-private and shall not become a
stable compiled-library ABI, C ABI, or other foreign interface.

DWARF shall describe the ordered source Tuple with size, alignment, and offsets
derived from the qualified x86-64 data layout. Because the LLVM 22 x86 backend
does not retain a directly described SSA aggregate as an inspectable O0 local,
named Tuple bindings shall use a target-aligned debug-only stack shadow and
`#dbg_declare`. This shall introduce no semantic aggregate storage, runtime,
allocator, foreign dependency, C/C++ runtime, other-language standard library,
or `topal-native/6` revision. This realizes
`TOPAL-COMPILER-TUPLE-RESULT-001` for compiler increment 3b2-b5f.

## TOPAL-COMP-TUPLE-DECISION-001 — Field-wise Tuple control-flow joins

The checked compiler model shall admit a common recursively composed Tuple
result across Boolean, ordered-comparison, `Comparison`, Enum, Optional, and
Result decisions when every leaf has the private representation admitted by
`TOPAL-COMP-TUPLE-RESULT-001`. It shall require identical complete action types
after existing conversions and shall preserve subject-once evaluation, rule
order, and exactly one selected delayed action at O0.

The backend shall recursively join every decomposed machine leaf with a
correctly typed LLVM `phi`, omitting a machine join only for Unit leaves. It
shall not construct an aggregate `phi`, evaluate an unselected action, or add
semantic aggregate storage, heap allocation, a runtime helper, a foreign
dependency, a C/C++ runtime, another-language standard library, or a
`topal-native/6` revision. Records admitted by
`TOPAL-COMP-RECORD-BOUNDARY-001` may occur recursively; unsupported Tuple leaves
shall remain rejected. This realizes `TOPAL-COMPILER-TUPLE-DECISION-001` for
compiler increment 3b2-b5g.

## TOPAL-COMP-TUPLE-PARAMETER-001 — Private positional-product parameters

The checked compiler model shall admit ordinary and static Tuple parameters
when every recursively nested leaf has the exact private representation
admitted by `TOPAL-COMP-TUPLE-RESULT-001`. It shall evaluate the argument once
and preserve interpreter-compatible call normalization: a unary candidate
matches the complete Tuple, a multi-parameter candidate matches its fields in
declaration order, and otherwise-applicable overloads remain source ordered.
It shall admit recursively nested Record fields covered by
`TOPAL-COMP-RECORD-BOUNDARY-001` and reject other unsupported Tuple leaves.

The Linux x86-64 backend shall pass each Tuple parameter as one recursively
nested, non-packed LLVM literal struct. Caller `insertvalue` construction,
callee `extractvalue` decomposition, definition, and calls shall all use the
same exact private `fastcc` prototype while LLVM performs target-specific
physical call lowering. Named Tuple parameters shall have target-exact DWARF
and remain inspectable in GDB at O0 through a target-aligned debug-only stack
shadow and `#dbg_declare`. A discarded Tuple parameter shall retain its
signature slot but shall have no generated decomposition, source binding, or
DWARF variable. This shall add no semantic aggregate storage, heap allocation,
runtime helper, foreign dependency, C/C++ runtime, other-language standard
library, public aggregate ABI, or `topal-native/6` revision. This realizes
`TOPAL-COMPILER-TUPLE-PARAMETER-001` for compiler increment 3b2-b5h.

## TOPAL-COMP-TYPE-VALUE-001 — Closed fundamental Type values

The checked compiler model shall recognize the seven fundamental Type values
defined by `TOPAL-ABSTRACTION-TYPE-VALUE-001`, preserve the enclosing `Type`
classifier and each exact identity, compare only canonical identities, and
retain the values through bindings, decomposed products, and scalar function
boundaries. Canonical display and DWARF/GDB shall expose the source type names.
User-defined Type values and runtime reflection remain explicitly unsupported.

The backend may lower the closed set to private tags, but shall not expose tag
numbers through a public ABI or use them as serialized library-metadata
identities. Future library metadata shall carry canonical semantic identities
independent of this target representation. No registry, allocation, foreign
type-information runtime, C/C++ runtime, other-language standard library, or
`topal-native/6` revision is permitted. This realizes
`TOPAL-COMPILER-TYPE-VALUE-001` for compiler increment 8a.

## TOPAL-COMP-LAYOUT-POLICY-001 — Closed external-layout policy values

The checked compiler model shall recognize `Little` and `Big` as `Endian`;
`ReadWrite`, `ReadOnly`, `WriteOnly`, and `Reserved` as `Access`;
`MostSignificantFirst` and `LeastSignificantFirst` as `BitOrder`; `Natural` and
`Packed` as `Packing`; `Declared` as `FieldOrder`; `AfterTag` and `Overlay` as
`PayloadPlacement`; and `NoLength` and `NoTerminator` as `LayoutPolicy`. It
shall preserve these seven nominal identities and declaration orders through
immutable bindings, decomposed products, same-type equality, canonical output,
DWARF, and GDB. Cross-family equality shall receive a stable checked type
diagnostic.

The Linux x86-64 backend may reuse its private declaration-ordered `i32` enum
tags. The tags shall remain compiler-selected representation rather than
semantic or serialized identities. This increment shall construct no external
layout, encode or serialize no data, designate no location or address, grant no
access authority, and establish no public, foreign, persistent, or compiled-
library ABI. Future library metadata shall use canonical semantic policy and
value identities independently of these tags. The executable shall require no
layout runtime, allocator, foreign dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, or `topal-native/6`
revision. This realizes `TOPAL-COMPILER-LAYOUT-POLICY-001` and
`TOPAL-LAYOUT-ENDIAN-001`, `TOPAL-LAYOUT-ACCESS-001`,
`TOPAL-LAYOUT-BIT-ORDER-001`, `TOPAL-LAYOUT-PACKING-001`,
`TOPAL-LAYOUT-FIELD-ORDER-001`, `TOPAL-LAYOUT-PAYLOAD-PLACEMENT-001`, and
`TOPAL-LAYOUT-ABSENCE-POLICY-001` for compiler increment 7b1.

## TOPAL-COMP-STATIC-INTROSPECTION-001 — Closed static introspection foundation

The checked compiler model shall admit `lang identity` and `lang view` for the
seven directly named closed fundamental v0.1 Type values. It shall retain an
object-kind-tagged canonical identity and a kind-preserving primitive Type-view
value, respectively. `lang context` shall retain `topal`, the active numeric v0.1
Version, and the exact empty feature set. All three result types shall remain
static-only: their bindings shall stay out of the runtime/root environments and
their use or containment at a runtime boundary shall receive a stable checked
diagnostic. Code generation shall erase them without an instruction, local,
DWARF type, descriptor, or executable metadata entry.

For directly named admitted fundamental Type operands, `lang same-object` and
`lang equivalent-type` shall compare canonical semantic identities during
checking and yield ordinary Boolean constants. Runtime operands and unadmitted
object kinds shall be rejected. `lang version` shall produce the ordinary
numeric Version for the active context with immutable Nat `major`, `minor`,
`patch`, and `build` components and canonical abbreviated display.

The Linux x86-64 backend may materialize this initial Version as a private
frame-local four-pointer header over the existing immutable Nat carrier. LLVM
shall own instruction and stack lowering for the target data layout. DWARF and
the bundled validating GDB renderer shall expose the same four-field Version
and canonical source spelling. The linked executable shall have no load-time
relocation, undefined symbol, needed library, reflection/Version runtime,
foreign runtime, C/C++ standard library, public/foreign/serialized/library
Version ABI, or `topal-native/6` revision. General introspection, other object
kinds and relations, later context changes, static-to-runtime conversion, and
Version operations and function boundaries shall remain unsupported. This
realizes `TOPAL-COMPILER-STATIC-INTROSPECTION-001`,
`TOPAL-INTRO-QUALIFIED-001`, `TOPAL-INTRO-STATIC-001`,
`TOPAL-INTRO-VIEW-001`, `TOPAL-INTRO-CONTEXT-001`, and
`TOPAL-INTRO-RELATION-001` for compiler increment 8a3.

## TOPAL-COMP-CAPABILITY-COMPOSE-001 — Closed static Capability composition

The checked compiler model shall recognize the six v0.1 atomic Capability
values `Equality`, `Ordering`, `Foldable`, `Membership`, `Indexed`, and `Keyed`.
It shall retain canonical alternatives of canonical atomic-promise
conjunctions, fold root-scope `and` as their cross-product, fold `or` as their
alternative union, remove duplicates, and validate the explicit `Capability`
classifier. These values shall be static-only. Root binding chains shall retain the
checked metadata, while non-root bindings, aggregate containment, functions and
other machine boundaries, unknown atoms, application, claims, and other
operators shall receive stable checked diagnostics.

Code generation shall erase every Capability binding and discard before LLVM,
including its source name, type, metadata, and debug entry. An exact
source-entry Capability result may emit only its canonical interpreter-matching
text through the existing Topal syscall writer; no Capability machine value may
be formed. LLVM IR and the executable shall contain no capability/evidence
runtime helper, dispatch, table, tag, descriptor, registry, allocation, or
DWARF Capability type. Native artifact tests shall retain zero undefined
symbols, needed libraries, and load-time relocations.

This increment shall add no foreign runtime, C/C++ runtime, other-language
standard library, public/foreign/serialized/library Capability ABI,
compiled-library metadata format, or `topal-native/6` revision. Future library
metadata shall use separately versioned canonical semantic identities rather
than any target representation. This realizes
`TOPAL-COMPILER-CAPABILITY-COMPOSE-001`,
`TOPAL-CAPABILITY-EVIDENCE-001`, `TOPAL-CAPABILITY-COHERENCE-001`, and
`TOPAL-CAPABILITY-COMPOSE-001` for compiler increment 8a4.

## TOPAL-COMP-FUNCTION-INTERFACE-001 — Closed direct function-interface conformance

For source-root v0.1 interfaces containing only uniquely named ordinary
function shapes over already admitted private native parameter and result
classifiers, the checked compiler shall retain the nominal `root.Name`
identity and canonical operation name/input/result metadata. A following
direct construction shall be validated before function collection and shall
provide exactly one ordinary function declaration for each role, with no
missing, additional, duplicate, or mismatched operation. Successful evidence
shall map each canonical role to its overload-qualified root declaration
identity and its optional explicit empty declared effect bound. An absent bound
shall remain absent rather than being mislabeled as inferred evidence for an
unchecked body.

Only the selected ordinary declarations shall enter LLVM, using the existing
direct module-private `fastcc` calls and LLVM-owned target argument lowering.
Interface shapes and evidence shall produce no instruction, value, namespace,
vtable, function pointer, indirect call, dispatcher, tag, descriptor,
allocation, symbol, relocation, DWARF type, or DWARF variable. DWARF/GDB shall
still expose the implementation subprogram, String parameter, source line,
direct caller frame, and canonical value. Native tests shall cover the shared
interpreter regression, exact model evidence, unknown/missing/additional/
duplicate/mismatched rejection, direct IR, exact output, undefined symbols,
needed libraries, relocations, metadata erasure, DWARF erasure, and GDB.

This increment shall add no foreign runtime, C/C++ runtime, other-language
standard library, public/foreign/serialized interface ABI, native-artifact
export/evidence claim, compiled-library metadata format, or `topal-native/6`
revision. Future compiled libraries shall publish these semantic identities,
shapes, effects, and declaration mappings through a separately versioned and
validated schema rather than reuse `fastcc`, LLVM types, private symbols, or
target layouts. Packaged/dynamic implementations, generators, non-root/message
contexts, v0.2 contracts, and cross-library consumption remain rejected. This
realizes `TOPAL-COMPILER-FUNCTION-INTERFACE-001`,
`TOPAL-INTERFACE-SHAPE-001`, and `TOPAL-INTERFACE-IMPLEMENTATION-001` for
compiler increment 8a5.

## TOPAL-COMP-PATTERN-001 — Positional product and discard inputs

For an admitted ordinary function with multiple scalar parameters, a prefix
application containing one positional product shall validate and pass its
fields as the declared parameter sequence in left-to-right order. Overload
selection shall use the complete flattened sequence without re-evaluating any
field.

A typed `_` parameter shall validate its argument classifier and occupy its
private machine-signature position but introduce no checked source binding,
generated-value binding, or DWARF variable. Other parameters retain their
source argument ordinals in debug information.

This requirement covers the admitted function-input case of
`TOPAL-TYPE-MATCH-001` and realizes `TOPAL-COMPILER-PATTERN-001` for compiler
increment 3b2-b3.

## TOPAL-COMP-OPTIONAL-001 — Optional values and control flow

The compiler shall construct `Some` and explicit or immediately contextual
`None` values for the admitted `Optional Int` and `Optional String` subset,
retain the nominal payload classifier through classified bindings and ordinary
function parameters/results, and preserve the value through a direct return or
machine-scalar decision join. The representation shall be an immutable opaque
pointer with its own validated Optional tag semantics under `topal-native/6`;
it shall not be reclassified as Result or exposed to a foreign aggregate ABI.

An Optional decision shall evaluate its subject once, load and bind a present
payload only in the selected `Some` action, execute only the selected action,
and require `Some` plus `None` coverage or a final `otherwise`. `Optional Int`
shall implement derived equality using canonical arbitrary-precision Int
equality. `Optional String` shall implement derived equality using exact
preserved-sequence String equality.

Construction, display, function passage, decisions, equality, DWARF, and GDB
shall use the same private header and Linux syscall-backed allocator without a
C/C++ runtime, standard library, load-time pointer relocation, or runtime type
lookup. This requirement covers `TOPAL-TYPE-OPTIONAL-CONSTRUCT-001`,
`TOPAL-TYPE-OPTIONAL-CONTEXT-001`, `TOPAL-TYPE-OPTIONAL-BOUNDARY-001`,
`TOPAL-DECISION-OPTIONAL-001`, and the admitted Int- and String-payload cases of
`TOPAL-TYPE-OPTIONAL-EQUALITY-001`. It realizes
`TOPAL-COMPILER-OPTIONAL-001` for compiler increment 3b2-b4.

## TOPAL-COMP-OPTIONAL-RATIONAL-001 — Exact Optional Rational values

The compiler shall extend the existing native Optional construction, contextual
absence, function passage and result, direct return, control-flow join,
decision, display, DWARF, and GDB paths to the admitted `Optional Rational`
payload. It shall preserve the existing opaque Optional header and the existing
canonical Rational payload pointer under `topal-native/6`, without a new public
aggregate, runtime type lookup, C/C++ runtime, standard library, or ABI revision.

Derived equality shall validate both Optional alternatives and call the exact
canonical Rational comparator only when both are `Some`; it shall not load an
absent payload. The equality shall also be available recursively to admitted
same-classifier positional-product equality. This requirement covers the
Rational-payload cases of `TOPAL-TYPE-OPTIONAL-CONSTRUCT-001`,
`TOPAL-TYPE-OPTIONAL-CONTEXT-001`, `TOPAL-TYPE-OPTIONAL-BOUNDARY-001`,
`TOPAL-DECISION-OPTIONAL-001`, and `TOPAL-TYPE-OPTIONAL-EQUALITY-001`. It
realizes `TOPAL-COMPILER-OPTIONAL-RATIONAL-001` for compiler increment
3b2-b4a.

## TOPAL-COMP-TUPLE-EQUALITY-001 — Derived positional-product equality

The compiler shall admit equality and inequality between same-classifier
positional products exactly when every field has an admitted canonical equality
lowering. The admitted recursive field set is Unit, Completed, Boolean, Int,
Nat, Rational, Comparison, ErrorCode, Character, String, payload-free source
Enum, `Optional Int`, `Optional Rational`, `Optional String`, and another
admitted positional product.
Both complete operands shall be evaluated once from left to right before their
corresponding fields are recursively compared, and inequality shall negate the
same all-fields-equal result.

An expression-local product shall remain a compiler aggregate of field values;
comparison shall require no allocation, runtime product header, or foreign
aggregate ABI. Canonical field conversion, general product passage across
machine signatures, and equality for further field classifiers remain outside
this increment and shall be rejected at the checked boundary. This requirement
covers the admitted same-classifier positional-product case of
`TOPAL-TYPE-EQUALITY-001` and realizes
`TOPAL-COMPILER-TUPLE-EQUALITY-001` for compiler increment 3b2-b5a.

## TOPAL-COMP-RECORD-001 — Anonymous record construction and selection

The checked compiler model shall admit anonymous products whose fields are all
labeled as structural Records under `TOPAL-TYPE-PRODUCT-001`. It shall reject a
duplicate label with `E-DUPLICATE-RECORD-FIELD`, evaluate field expressions once
from left to right, retain source order for canonical display, and retain a
canonical label-to-classifier map for static identity. Static field selection
shall return the selected already-evaluated field with its exact classifier and
shall diagnose an absent label with `E-NO-SUCH-RECORD-FIELD` at that label.

Expression-local Records shall lower as decomposed LLVM values without an
allocation, native header, generated record runtime, C/C++ runtime, standard
library, or ABI revision. `TOPAL-COMP-RECORD-BOUNDARY-001` separately admits
private aggregate function passage, control-flow joins, and truthful debug-only
storage. Persistent semantic Record storage remains rejected. This requirement
realizes `TOPAL-COMPILER-RECORD-001` for compiler increment 3b2-b5b.

## TOPAL-COMP-RECORD-BOUNDARY-001 — Order-preserving private Record boundaries

The checked compiler model shall parse and retain closed structural Record
classifiers on ordinary and static parameters and results. It shall compare
classifiers by canonical label-to-type maps, recursively admit Tuple and Record
fields with exact private representations, preserve each value's independent
construction order, evaluate an argument once, and retain source-ordered
candidate selection.

The Linux x86-64 backend shall lower each admitted Record boundary to one
non-packed LLVM literal struct. Values shall appear in canonical label order,
followed by one `i32` canonical-field index per source display position. Caller
`insertvalue`, callee `extractvalue`, results, definitions, and calls shall use
the same exact private `fastcc` type while LLVM performs target-specific
physical call lowering. Every admitted decision family shall join canonical
fields recursively and order indexes individually with LLVM `phi` nodes, with
no aggregate `phi` or eager action execution.

DWARF shall expose semantic named fields at target-derived offsets and account
for the private order suffix in the complete size without inventing source
members. Named Record bindings and parameters shall remain inspectable in GDB
at O0 through a target-aligned debug-only stack shadow and `#dbg_declare` where
needed. This shall add no persistent semantic aggregate storage, heap
allocation, Record runtime helper, foreign dependency, C/C++ runtime,
other-language standard library, public aggregate ABI, or `topal-native/6`
revision. This realizes `TOPAL-COMPILER-RECORD-BOUNDARY-001` for compiler
increment 3b2-b5i.

## TOPAL-COMP-STRUCTURAL-COMPARISON-001 — Derived structural comparison

The checked compiler model shall derive Tuple total ordering recursively when
every corresponding field has admitted ordering. It shall evaluate both
complete operands once from left to right, apply canonical Int/Nat/Rational
conversion per field, and lower lexicographic comparison so later field
comparisons execute only after every preceding field compares Equal. The same
three-way result shall drive `<`, `>`, `<=`, `>=`, and `<=>`.

The compiler shall derive anonymous Record equality and inequality for the same
canonical label set when corresponding label values have admitted equality. It
shall align fields by label independently of construction order and apply
admitted canonical field conversions. Different shapes or unsupported field
pairs shall produce `E-NO-APPLICABLE-OVERLOAD`; conversions hidden in a
non-decomposable aggregate shall remain explicitly unsupported.

LLVM lowering shall reuse decomposed Tuple and Record values, existing exact
comparators, `br`, and `phi`, without aggregate allocation, a structural runtime,
C/C++ runtime, standard library, or native ABI revision. Boolean and Comparison
results shall reuse their existing DWARF/GDB paths. This requirement covers the
admitted structural cases of `TOPAL-TYPE-EQUALITY-001`,
`TOPAL-TYPE-ORDERING-001`, and `TOPAL-NUM-INT-RATIONAL-CONVERT-001`; it realizes
`TOPAL-COMPILER-STRUCTURAL-COMPARISON-001` for increment 3b2-b5c.

## TOPAL-COMP-RECONSTRUCT-001 — Immutable record reconstruction

The checked compiler model shall admit `with` reconstruction of a structural
Record under `TOPAL-TYPE-RECONSTRUCT-001`. It shall evaluate the complete base
once before replacements, then evaluate replacements once from left to right.
Each replacement label shall be unique and present in the base, and its value
shall retain the original field classifier after any admitted exact canonical
conversion. The result shall preserve the base field order and unreplaced
values without changing the original Record.

LLVM lowering shall replace named values in the existing decomposed Record and
shall introduce no allocation, generated reconstruction runtime, public ABI,
C/C++ runtime, or standard-library dependency. Existing scalar DWARF values
projected from the original and reconstructed Records shall remain inspectable;
the compiler shall continue to omit the nonexistent aggregate storage from
DWARF. The compiler shall retain the interpreter diagnostics
`E-RECONSTRUCT-NON-RECORD`, `E-DUPLICATE-RECONSTRUCTION-FIELD`,
`E-NO-SUCH-RECORD-FIELD`, and `E-TYPE-MISMATCH` at the checked boundary. This
requirement realizes `TOPAL-COMPILER-RECONSTRUCT-001` for increment 3b2-b5d.

## TOPAL-COMP-STRING-UTF8-BYTE-COUNT-001 — Prospective UTF-8 byte count

For an admitted plain String, the compiler shall evaluate
`text byte-count Utf8` once, load the preserved UTF-8 byte length from the
immutable native descriptor, and return the equal canonical arbitrary-
precision Int. The operation shall neither use the cached display spelling nor
modify, normalize, encode, or reclassify the String.

The runtime shall convert the complete unsigned target length to normalized
base-2^32 Int limbs without truncation, a fixed source-value width, or a
foreign conversion helper. It shall allocate only through the Linux syscall
platform boundary and introduce no C/C++ runtime or standard-library
dependency. This requirement covers `TOPAL-TYPE-CALL-001` and
`TOPAL-STRING-UTF8-BYTE-COUNT-001`; it realizes
`TOPAL-COMPILER-STRING-UTF8-BYTE-COUNT-001` for compiler increment 4a.

## TOPAL-COMP-STRING-EQUALITY-001 — Exact String equality

The compiler shall evaluate each admitted String equality operand once and
compare the complete preserved Unicode sequence. Its valid UTF-8 native
representation shall implement this by comparing the stored preserved-byte
lengths and then corresponding bytes in order, without inspecting display
spelling, normalizing either operand, consulting locale state, or calling a
foreign String routine.

Derived `Optional String` equality shall validate both Optional alternatives,
compare payloads only when both are `Some`, make two `None` alternatives equal,
and make different alternatives unequal without loading an absent payload.
Inequality shall be the Boolean negation of the same equality result. This
requirement covers the String case of `TOPAL-TYPE-EQUALITY-001` and the admitted
String-payload case of `TOPAL-TYPE-OPTIONAL-EQUALITY-001`; it realizes
`TOPAL-COMPILER-STRING-EQUALITY-001` for compiler increment 4b1.

## TOPAL-COMP-STRING-CONSTRUCTION-001 — String construction and emptiness

The compiler shall construct the unique empty plain String value and compose
adjacent source literal primaries into one preserved sequence as required by
the frontend semantics. Dynamic plain concatenation shall evaluate operands
once from left to right, send target-length overflow through the explicit
platform storage-failure path, allocate exact byte storage through the Linux
syscall platform, and copy both valid UTF-8 sequences in order without
normalization or content changes. The copy intrinsic shall guarantee no
external-function lowering. A zero-length result shall not depend on a
zero-length mapping.

String emptiness shall read the preserved-byte length and return true exactly
for the empty sequence. A dynamic descriptor may omit the literal display cache;
the runtime shall then emit the canonical ordinary or shortest collision-free
tagged spelling directly from preserved bytes. Compiled output and the GDB
renderer shall agree, including values containing quote-and-tag collisions.
These operations shall not use a C/C++ runtime, standard library, foreign
allocator, locale, or Unicode transformation routine.

This requirement covers `TOPAL-TYPE-CALL-001`, `TOPAL-STRING-EMPTY-001`,
`TOPAL-STRING-LITERAL-COMPOSE-001`, `TOPAL-STRING-CONCAT-001`, and
`TOPAL-STRING-EMPTY-PREDICATE-001`; it realizes
`TOPAL-COMPILER-STRING-CONSTRUCTION-001` for compiler increment 4b2.

## TOPAL-COMP-CHARACTER-001 — Retained static Character evidence

For a closed String expression known to the checked model, the compiler shall
use the selected language context's pinned Unicode segmentation to admit the
Character constraint exactly when the preserved sequence has one extended
grapheme cluster. It shall report `E-CHARACTER-CLASSIFIER` with the observed
count for a closed invalid value. Dynamic validation shall remain rejected at
the checked boundary until its explicit Result and freestanding runtime path
are implemented.

Character bindings and ordinary function parameters/results shall retain their
classifier, while explicit or implicit forgetting to String shall emit no
conversion and preserve every scalar. Character equality and derived product
field equality shall call the existing exact String comparator. DWARF shall
expose a distinct Character typedef over the same immutable String descriptor,
and the GDB renderer shall display its preserved sequence. This requirement
covers `TOPAL-TYPE-CONSTRAINT-VALIDATE-001`,
`TOPAL-STRING-CHARACTER-CLASSIFIER-001`,
`TOPAL-STRING-FROM-CHARACTER-001`, and the Character case of
`TOPAL-TYPE-EQUALITY-001`; it realizes `TOPAL-COMPILER-CHARACTER-001` for
compiler increment 4b3a without a C/C++ runtime, standard library, foreign
Unicode implementation, or `topal-native/6` revision.

## TOPAL-COMP-CHARACTER-OBSERVATION-001 — Closed Character observations

For a String whose complete preserved sequence is known to the checked model,
the compiler shall evaluate `character-count` and String `entry-count` with the
selected context's pinned extended-grapheme segmentation and materialize the
equal canonical arbitrary-precision Int. When an exact Int index is also known,
it shall evaluate `character-at` with that same segmentation and materialize an
`Optional Character`: the complete cluster in `Some`, or `None` for a negative
or out-of-range index.

The fold shall be mandatory frontend semantics for this admitted subset at
`-O0`, independent of LLVM optimization. A produced `Optional Character` shall
reuse the existing private Optional header and immutable String-descriptor
payload through ordinary function passage, decisions, display, DWARF, and GDB.
General Optional-Character construction/equality and dynamic String or index
observations remain rejected until reusable constraint evidence and a
Topal-owned freestanding Unicode runtime are implemented.

This requirement covers `TOPAL-STRING-CHARACTER-COUNT-001`,
`TOPAL-STRING-ENTRY-COUNT-001`, `TOPAL-STRING-CHARACTER-AT-001`,
`TOPAL-TYPE-OPTIONAL-BOUNDARY-001`, and `TOPAL-DECISION-OPTIONAL-001`; it
realizes `TOPAL-COMPILER-CHARACTER-OBSERVATION-001` for compiler increment
4b3b without a C/C++ runtime, standard library, host locale or Unicode table,
or `topal-native/6` revision.

## TOPAL-COMP-STRING-CHARACTERS-FOREACH-001 — Closed Character traversal

The checked compiler shall admit a root `foreach` directly over `characters
text` when the complete plain String is known during checking and the action is
capture-free, binds Character, and produces Unit. It shall evaluate the source
String exactly once, use the selected language context's pinned extended-
grapheme segmentation, invoke the action once per complete Character in
preserved order, resume with Unit after each
action, and return Unit without invoking the action for empty input. The source
String shall remain immutable. The statement may bind its Unit result with an
optional exact Unit classifier or leave the result unnamed.

The Linux x86-64 backend shall expand the statically known Character sequence
through existing immutable String descriptors and inline the checked action in
order. This is mandatory O0 source-semantic lowering, not an optimization-pass
result. DWARF/GDB shall preserve the Character action binding, including a
debug-only pointer shadow for an otherwise instruction-free action. The
lowering shall create no semantic traversal collection, Generator object or
token, generic Generator runtime, callback, indirect call, host Unicode or
locale dependency, C/C++ runtime, other-language standard library, needed
library, dynamic relocation, public/library Generator ABI, or
`topal-native/6` revision. Dynamic Strings, function-returned Character
generators outside `TOPAL-COMP-STRING-CHARACTERS-RESULT-001`, other
non-specializable Character generators, and captures remain rejected. Future
library metadata shall carry canonical Generator, pinned-segmentation,
action-evidence, ownership, and target-adapter identities rather than this
specialization. This realizes
`TOPAL-COMPILER-STRING-CHARACTERS-FOREACH-001`,
`TOPAL-STRING-CHARACTERS-COLLECT-001`, and
`TOPAL-STRING-CHARACTERS-FOREACH-001` for compiler increment 4b3d-k.

## TOPAL-COMP-STRING-CHARACTERS-GENERATOR-001 — Named closed traversal

The checked compiler shall admit `characters text` as an exact
`Generator Character Unit Unit` when the complete plain String is known. A root
binding may state that classifier and shall evaluate the source String exactly
once while creating a fresh linear value. The checked model may retain the
pinned ordered Character sequence as compile-session provenance, but
construction shall invoke no action and the generated observation token shall
not serve as continuation state.

Root foreach shall transfer one locally bound value into the existing closed
Character traversal, preserve its action order and Unit result, and mark the
source consumed. Later source use shall report `E-GENERATOR-CONSUMED` rather
than copy or restart the traversal. Unconsumed root bindings shall remain
rejected; the narrow transferred-parameter close is governed by
`TOPAL-COMP-STRING-CHARACTERS-CLOSE-001`. DWARF/GDB shall describe the local
binding as `Generator Character Unit Unit` through a compiler-private token.

The token and compiler-held provenance shall require no Generator object,
continuation-state allocation, generic Generator runtime, callback, indirect
call, host Unicode/locale dependency, C/C++ runtime, other-language standard
library, needed library, dynamic relocation, public/library Generator ABI, or
`topal-native/6` revision. General function result transfer outside
`TOPAL-COMP-STRING-CHARACTERS-RESULT-001`, general parameter use, and external
boundaries remain rejected pending canonical Generator, segmentation,
operation/evidence, ownership, close, and target-adapter metadata. This realizes
`TOPAL-COMPILER-STRING-CHARACTERS-GENERATOR-001`,
`TOPAL-STRING-CHARACTERS-COLLECT-001`,
`TOPAL-STRING-CHARACTERS-FOREACH-001`,
`TOPAL-STRING-CHARACTERS-GENERATOR-001`,
`TOPAL-STRING-CHARACTERS-CLASSIFIER-001`, and
`TOPAL-STRING-CHARACTERS-LINEAR-001` for compiler increment 4b3d-l.

## TOPAL-COMP-STRING-CHARACTERS-COLLECT-001 — Closed String reconstruction

The checked compiler shall admit direct `characters text collect String` when
the complete plain String is known. It shall evaluate `text` exactly once,
retain the selected language context's pinned ordered Character segmentation
as checking evidence, consume the fresh traversal, and return a plain String
with exactly the source's preserved scalar sequence. Empty input shall return
`empty String`.

The Linux x86-64 backend may forward the existing immutable source String
descriptor because this exact unchanged traversal reconstructs that source.
This shall be mandatory O0 semantic lowering, not an LLVM optimization or a
rule for transformed traversals. DWARF/GDB shall expose the result as String.

The lowering shall create no intermediate List or Generator object,
concatenation loop, generic traversal runtime, callback, indirect call, host
Unicode/locale dependency, C/C++ runtime, other-language standard library,
needed library, dynamic relocation, public/library Generator ABI, or
`topal-native/6` revision. Dynamic Strings, transformed traversals, stored
Generator collection, and external boundaries remain rejected pending
generated Topal Unicode support and canonical traversal, operation/evidence,
ownership, and target-adapter metadata. This realizes
`TOPAL-COMPILER-STRING-CHARACTERS-COLLECT-001` and
`TOPAL-STRING-CHARACTERS-COLLECT-001` for compiler increment 4b3d-m.

## TOPAL-COMP-STRING-CHARACTERS-CLOSE-001 — Owned parameter close

The checked compiler shall admit an ordinary called function with exactly one
named `Generator Character Unit Unit` parameter, a Unit result, and a
statement-free Unit body that leaves the parameter untraversed. Passing a
locally bound closed Character generator shall transfer and consume the caller
binding. Function exit shall deliver intrinsic close to that owned built-in
continuation and return Unit without another yield. The checked model shall
retain an explicit close operation.

Because the admitted built-in representation allocates no continuation object
or live state, Linux x86-64 code generation shall lower close to no runtime
action after accepting the compiler-private observation token. The private
function shall use LLVM `fastcc` for the `i32` token rather than hard-coded
System V register placement. DWARF/GDB shall expose the Generator parameter
through debug-only storage when no semantic instruction preserves its location.

The specialization shall add no generic Generator runtime, close dispatcher,
allocation, callback, indirect call, unwind dependency, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or `topal-native/6`
revision. Return of the parameter, traversal beyond
`TOPAL-COMP-STRING-CHARACTERS-PARAMETER-001`, multiple/additional parameters,
static functions, general close handling, and external boundaries remain
rejected pending canonical Generator classifier, state, ownership,
close-domain/provenance, and target-adapter metadata. This realizes
`TOPAL-COMPILER-STRING-CHARACTERS-CLOSE-001` and
`TOPAL-STRING-CHARACTERS-CLOSE-001`, plus the applicable ownership-transfer
obligations of `TOPAL-STRING-CHARACTERS-GENERATOR-001` and
`TOPAL-STRING-CHARACTERS-PARAMETER-001`, for compiler increment 4b3d-n.

## TOPAL-COMP-STRING-CHARACTERS-PARAMETER-001 — Specialized parameter traversal

The checked compiler shall admit an ordinary called function with exactly one
named `Generator Character Unit Unit` parameter and a Unit result when its
executable body is exactly one foreach over that parameter with a capture-free
Character-to-Unit action. The top-level call argument shall be a locally bound
closed Character generator with retained provenance, and the call shall
transfer and consume the caller binding once.

Each call shall create a private specialization receiving only that argument's
pinned ordered Character sequence in compile-session provenance. The caller
shall evaluate the source String once. The callee shall expand the action once
per Character in order, resume with Unit, exhaust the continuation, and return
Unit without close delivery. Distinct calls shall not share their retained
sequences.

The Linux x86-64 private function shall accept the existing `i32`
observation/ownership token through LLVM `fastcc`, without using it as cursor or
continuation state or hard-coding System V register placement. DWARF/GDB shall
expose the Generator parameter and Character binding.

The specialization shall add no Generator object/runtime, state allocation,
dispatcher, callback, indirect call, host Unicode/locale dependency, C/C++
runtime, other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or `topal-native/6`
revision. Dynamic/transformed provenance, general function results outside
`TOPAL-COMP-STRING-CHARACTERS-RESULT-001`, multiple/additional parameters,
nested calls, other bodies, and external boundaries remain rejected pending
canonical Generator, segmentation, action-evidence, ownership, and
target-adapter metadata. This realizes
`TOPAL-COMPILER-STRING-CHARACTERS-PARAMETER-001`,
`TOPAL-STRING-CHARACTERS-FOREACH-001`, and the relevant transfer obligations of
`TOPAL-STRING-CHARACTERS-GENERATOR-001` and
`TOPAL-STRING-CHARACTERS-PARAMETER-001` for compiler increment 4b3d-o.

## TOPAL-COMP-STRING-CHARACTERS-RESULT-001 — Specialized function result

The checked compiler shall admit an ordinary nonrecursive called function with
exactly one named String parameter and result classifier
`Generator Character Unit Unit` when its body has no statements and returns
exactly `characters parameter`. Its top-level call argument shall have a closed
exact String value. Function exit shall transfer the fresh continuation without
close delivery, and the caller shall bind and consume the result exactly once
through an admitted Character traversal.

Every call shall create a distinct private specialization. The compiler shall
retain that argument's pinned ordered Character sequence beside its private
symbol only for checking. The caller shall evaluate the String once; the callee
shall observe its existing immutable descriptor and return the existing private
Generator token; caller traversal shall receive only that call's sequence.

On Linux x86-64, LLVM `fastcc` shall select the private String argument and
`i32` Generator-result placement without hard-coded System V registers.
DWARF/GDB shall expose the String parameter, Generator result classifier/value,
caller Character binding, and call frames.

The specialization shall add no Generator object/runtime, state allocation,
dispatcher, callback, indirect call, host Unicode/locale dependency, C/C++
runtime, other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or `topal-native/6`
revision. Dynamic/transformed arguments, anonymous/static/recursive/nested or
multi-parameter functions, other result bodies, unbound results, general close
handling, and external boundaries remain rejected pending canonical Generator,
segmentation, construction evidence, ownership, close, and target-adapter
metadata. This realizes `TOPAL-COMPILER-STRING-CHARACTERS-RESULT-001`,
`TOPAL-STRING-CHARACTERS-RESULT-001`, and the relevant linear/traversal
obligations of `TOPAL-STRING-CHARACTERS-GENERATOR-001` and
`TOPAL-STRING-CHARACTERS-FOREACH-001` for compiler increment 4b3d-p.

## TOPAL-COMP-UNICODE-FOLD-001 — Closed pinned-Unicode operations

When the checked model knows a String's complete preserved sequence, the
compiler shall evaluate universal `upper`, `lower`, full `case-fold`, NFC
normalization, and NFD normalization through the selected language context's
pinned Unicode implementation. It shall materialize the exact plain String
result without changing the source value. It shall also evaluate closed
`canonically-equals` operands through the same pinned data and materialize the
normative Boolean without normalizing either operand in place.

This frontend evaluation shall be mandatory at `-O0` and independent of LLVM
optimization. Result Strings shall use the existing immutable descriptor and
debug/display paths. Dynamic operands shall remain rejected until a Topal-owned
freestanding Unicode runtime is implemented; the binary shall gain no host
Unicode table, locale service, C/C++ runtime, standard library, or native ABI
revision.

This requirement covers `TOPAL-STRING-UPPER-001`, `TOPAL-STRING-LOWER-001`,
`TOPAL-STRING-CASE-FOLD-001`, `TOPAL-STRING-NORMALIZE-NFC-001`,
`TOPAL-STRING-NORMALIZE-NFD-001`, and
`TOPAL-STRING-CANONICAL-EQUALITY-001`; it realizes
`TOPAL-COMPILER-UNICODE-FOLD-001` for compiler increment 4b3c.

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
a C/C++ runtime, or introduce load-time pointer relocations. Unbounded forms
and collection selection remain outside this increment. Exact Int and Rational
infinity endpoints are admitted only under `TOPAL-COMP-INFINITY-001`.

This requirement covers `TOPAL-RANGE-BOUNDS-001`,
`TOPAL-RANGE-MEMBERSHIP-001`, `TOPAL-RANGE-RATIONAL-001`,
`TOPAL-RANGE-CLASSIFIER-001`, `TOPAL-RANGE-INTERSECTION-001`,
`TOPAL-RANGE-EMPTY-001`, and `TOPAL-RANGE-BOUND-001`. It realizes
`TOPAL-COMPILER-RANGE-001` for compiler increment 2d-a.

## TOPAL-COMP-INFINITY-001 — Contextual exact infinities

The checked compiler shall admit immediate root bindings classified as `Int`
with either exact infinity and as `Nat` with positive infinity. It shall retain
the source classifier and direction and admit canonical output, equality,
ordered predicates, three-way comparison, and explicitly bounded `Range Int`
construction, membership, intersection, emptiness, and bound observation. The
same closed root scope shall admit either infinity classified as `Rational`,
the corresponding comparisons and `Range Rational` operations, plus the
canonical embedding of a finite Int comparison value or endpoint. Bare
constants, negative Nat infinity, implicit cross-domain Int/Rational infinity
conversion, and all function, persistent, serialized, public, and
compiled-library infinity boundaries shall remain explicitly unsupported.
Closed root-scope negation, absolute value, addition, subtraction, and
multiplication shall admit the statically proved total cases of
`TOPAL-NUM-INFINITY-ARITHMETIC-001`; statically evident indeterminate cases
shall be diagnostics. When one infinity is multiplied by a finite factor whose
zero-ness is dynamic, the existing arithmetic Result ABI shall return code
`indeterminate` with source provenance for zero and the correctly signed
infinity for nonzero. Other dynamic indeterminate Results, division,
remainder, power, and directional-zero arithmetic remain explicitly
unsupported.

The Linux x86-64 backend may lower these closed root-local values to immutable
executable-private Int sentinels carrying reserved positive- and negative-
infinity tags with zero limbs. The checker shall prevent either sentinel from
crossing a private machine signature, so the finite Int representation and
`topal-native/6` function ABI remain unchanged. Runtime unary arithmetic,
addition, subtraction, multiplication, comparison, zero testing, and output
shall branch on the sentinel before finite sign, length, or limb handling.
Dynamic multiplication helpers shall validate the infinity operand before
Result construction, and a violated checked invariant shall fail closed. The
backend shall append those helpers only when the checked program contains this
fallible operation; unrelated O0 programs shall not compile or carry them. The
existing opaque Range pointer representation shall retain the endpoints. All
behavior shall be mandatory at O0 and use only the Topal-owned Linux syscall
runtime, without a
C/C++ runtime, other-language standard library, foreign allocator, undefined
helper, needed library, dynamic relocation, public ABI, or native ABI revision.

A Rational infinity shall use a run-time-constructed private Rational header
whose numerator is the matching Int sentinel and whose denominator is canonical
one. Rational construction, arithmetic normalization, comparison, display, and
debugging shall validate the sentinel/wrapper invariant before finite
greatest-common-divisor or cross-multiplication logic. Construction at run time
shall preserve the no-loader static-PIE relocation invariant; the finite
Rational representation and machine ABI remain unchanged.

DWARF and the bounded validating GDB renderer shall expose truthful `Int`,
`Nat`, `Rational`, `Range Int`, and `Range Rational` source values and reject
malformed sentinel or Rational-wrapper state.
Tests shall use shared interpreter regressions, compare exact output, inspect
checked and LLVM lowering, validate the complete corpus,
freestanding ELF, DWARF and GDB, and record separate interpreter execution plus
compiler build/run resource baselines. Future compiled-library metadata shall
describe language and numeric-domain revisions, infinity direction,
constraints, operations and indeterminate failures, ownership, debug
provenance, and native adapters independently of LLVM types, tags, headers, and
symbols. This realizes `TOPAL-COMPILER-INFINITY-001`,
`TOPAL-NUM-INFINITY-001`, `TOPAL-NUM-INFINITY-ARITHMETIC-001`, the infinity cases of `TOPAL-NUM-NAT-001`,
`TOPAL-NUM-COMPARE-001`, `TOPAL-NUM-THREE-WAY-COMPARE-001`, and the applicable
`TOPAL-RANGE-*` rules for compiler increments 2c-c1 through 2c-c4, 2d-b1, and
2d-b2.

## TOPAL-COMP-DECISION-001 — Comparison decisions

The compiler shall lower ordered comparison-matcher tables over admitted exact
numeric subjects to explicit source-ordered control flow with one subject
evaluation, branch-local matcher operand evaluation, first-match selection, a
required final `otherwise`, and one selected action. It shall also lower
complete decisions over the closed `Comparison` alternatives `Less`, `Equal`,
and `Greater`. Compatible machine-scalar action values shall merge through
typed SSA values.

Mixed Int/Rational matcher operands shall use the same canonical conversion as
ordinary exact comparisons. Lowering shall not evaluate a later matcher or any
unselected action and shall not call a runtime decision dispatcher.

This requirement covers `TOPAL-DECISION-COMPARISON-001`,
`TOPAL-DECISION-OPERAND-EXPRESSION-001`, and the language-defined `Comparison`
case of `TOPAL-DECISION-ENUM-001`. It realizes
`TOPAL-COMPILER-DECISION-001` for compiler increment 2c-a.

## TOPAL-COMP-CONSTRAINT-VALUE-001 — Named constraint observation values

The checked compiler model shall admit root named constraints over supported
primitive bases and retain the name, base classifier, predicate parameter, and
checked Boolean predicate as semantic metadata. A separately named root binding
classified as `Constraint` shall receive its binding identity while retaining
the source constraint's base and predicate. Captures shall be diagnosed as
unsupported rather than silently erased.

LLVM lowering shall carry only a deterministic module-private i32 identity tag,
with matching canonical `<Constraint name>` output and `Constraint` DWARF
enumerators. Native and GDB tests shall cover constructed and classified-copy
identities, checked Boolean result, exact shared-interpreter output, undefined
symbols, needed libraries, relocations, local types, values, and source frame.
The tag shall not dispatch the predicate or become a public ABI/library key.

Constraint application/evidence, capturing predicates, function or
persistent/public aggregate machine boundaries, and public compiled-library
constraint identities remain rejected. This shall add no allocation, constraint
runtime, foreign dependency, C/C++ runtime, other-language standard library, or
`topal-native/6` revision. This realizes `TOPAL-COMPILER-CONSTRAINT-VALUE-001`,
`TOPAL-ABSTRACTION-CONSTRAINT-CLASSIFIER-001`, and the construction-only portion
of `TOPAL-TYPE-CONSTRAINT-001` for compiler increment 8a1.

## TOPAL-COMP-CONSTRAINT-VALIDATE-001 — Int constraint validation

The checked compiler shall apply an admitted named Int constraint to a closed
exact operand by evaluating the retained predicate, diagnosing a known failure,
and giving an accepted unchanged Int value a distinct refined classifier.
Equality, ordering, and arithmetic shall deliberately forget this evidence and
reuse the canonical base operations. Unknown Int operands shall evaluate that
same predicate exactly once in generated code and use the existing Result/Error
representation with `root.Name(Int)`, `out-of-range`, and source provenance.

LLVM lowering shall bind the unchanged operand into a private predicate
environment and use explicit branches for success and failure. The success
payload shall be the original Int pointer. Refined locals shall use a DWARF
typedef over Int and a debug-only stack shadow so GDB preserves both source type
and value at O0. Native tests shall cover static success/rejection, base mismatch,
dynamic success/failure, equality, ordering, arithmetic, exact shared output,
IR paths, freestanding ELF properties, GDB values/types, and frames.

This shall add no duplicate numeric object, predicate dispatcher, constraint
runtime, foreign dependency, C/C++ runtime, other-language standard library,
public evidence ABI, or `topal-native/6` revision. Other bases, captured or
dependent predicates, evidence across function or persistent/public aggregate
machine boundaries, and dynamic constraint identities remain rejected. This
realizes `TOPAL-COMPILER-CONSTRAINT-VALIDATE-001` and
`TOPAL-TYPE-CONSTRAINT-VALIDATE-001` for compiler increment 8a2.

## TOPAL-COMP-NATIVE-SERIALIZATION-001 — Closed canonical native streams

The checked compiler shall admit both version-selected forms of `lang
serialize` and `lang deserialize` for compiler-created, authority-free streams
whose complete value is statically known as literal Unit or Boolean, exact Int,
known String, or one direct Tuple or Record whose fields are those supported
scalar values. It shall derive the bytes
through the shared canonical protocol implementation, using Topal protocol 1.0,
the selected language identity and revision, target little-endian value
encoding, declaration-ordered Record fields, and one finite value event.

Serialization shall still evaluate the source value exactly once at O0. The
Linux x86-64 backend shall retain the canonical expected bytes in immutable
executable storage, copy them into private Topal-owned stream storage, and
construct a Topal-owned private descriptor containing the copy's address and
byte count. The published copy shall thereafter be treated as immutable. Before
`lang deserialize` exposes the retained value, generated code shall validate the
descriptor length and every copied stream byte against the compile-time-validated
canonical bytes. A mismatch shall take the compiler-runtime corruption exit. The
implementation shall not replace this mandatory work with an LLVM optimization
or call a host serialization library.

Dynamic Boolean, Int, and String values, nested or indirect aggregate values, other
returnable classifiers, arbitrary or externally supplied streams, and
serialization across function, persistent, public, or library boundaries
remain rejected with a stable compiler diagnostic. A future compiled-library
interface carrying native serialization shall identify the protocol revision,
language identity and revision, canonical type identities and schemas, field
order, byte-order contract, and authority profile independently of the private
descriptor layout, LLVM type, and native symbol spelling.

Tests shall compile the unchanged interpreter regression, compare exact output,
inspect the checked stream and O0 LLVM, validate the shared corpus and separate
resource baselines, and verify canonical magic, runtime validation, freestanding
ELF, DWARF, GDB, undefined symbols, needed libraries, and relocations. The GDB
renderer shall bound reads and validate stream magic before rendering the byte
count. This shall add no foreign dependency, C/C++ runtime, other-language
standard library, public ABI, or `topal-native/6` revision. This realizes
`TOPAL-COMPILER-NATIVE-SERIALIZATION-001` and `TOPAL-SER-SCOPE-001` through
`TOPAL-SER-DESER-001` for compiler increment 8b1.

## TOPAL-COMP-TASK-DIRECT-001 — Closed direct task transactions

The checked compiler shall admit the unchanged task-declaration-order
regression as compiler increment 7b2. It shall retain the nominal task
classifier and definition identity, queue bound, deterministic immediate-FIFO
scheduler policy, one private Nat state-field schema, lifecycle signatures,
ordinary handler names/kinds/signatures, discarded-context evidence, and a
stable source-ordered transaction identity for every admitted send. The
admitted handlers are one exact Nat initializer, Nat-addition Unit events,
current-Nat requests with an empty error vocabulary, and an optional declared
no-op terminate handler. Other state schemas, handler bodies, observable
MessageContext, streams outside `TOPAL-COMP-TASK-STREAM-001`, overlapping
delivery, queue overflow, and termination delivery remain rejected with stable
diagnostics.

Construction shall evaluate the initial Nat once and allocate a fresh
Topal-owned instance with a distinct nonzero identity, active lifecycle tag,
and private state pointer. Immediate event delivery shall load the current
state, compute arbitrary-precision Nat addition, and commit the replacement
before a following request loads it. Since this closed root scheduler cannot
overlap sends and every admitted context is explicitly discarded, generated
O0 code may erase the unobservable context/transaction carrier and unused queue
storage after the checked model has retained their semantic identities. This is
a compiler semantic lowering and shall not depend on LLVM optimization.

The Linux x86-64 backend shall use only the Topal mmap allocator, exact Int/Nat
runtime, and direct private helpers. The instance layout and helpers shall not
cross function, public, persistent, serialized, foreign, or compiled-library
boundaries. LLVM shall remain responsible for physical target layout; no
System V argument placement shall be encoded in the frontend, and no foreign
dependency, C/C++ runtime, or other-language standard library may be added.

DWARF and the bundled GDB renderer shall expose and safely render the nominal
task type, identity, lifecycle state, and named private Nat state at the request
source frame. Functional evidence shall include checked-model metadata and
rejection tests, ordered O0 LLVM, exact interpreter differential, the complete
shared corpus, freestanding ELF/DWARF/GDB inspection, and separate build/run
resource baselines. A future compiled-library format shall carry versioned task
and definition identities, queue/scheduler policy, state schema, complete
handler/context/transaction signatures, effects, authorities, ownership,
termination, debug provenance, and native adapter requirements independently of
private LLVM types, object layouts, and symbol names. This realizes
`TOPAL-COMPILER-TASK-DIRECT-001`, `TOPAL-TASK-DEFINITION-001`,
`TOPAL-TASK-HANDLER-001`, `TOPAL-TASK-STATE-001`,
`TOPAL-TASK-LIFECYCLE-001`, and `TOPAL-TASK-MESSAGE-001` for increment 7b2.

## TOPAL-COMP-TASK-STREAM-001 — Closed one-yield task stream transaction

The checked compiler shall admit the unchanged task-message-transactions
regression as compiler increment 7b3. In addition to the direct-task metadata,
it shall retain an ordinary stream handler with discarded MessageContext and
Unit payload, Nat yield, Unit resumption, `Result (Unit, ())` final return, the
owning task instance, and one stable source-ordered transaction identity. The
admitted body is exactly one yield of the current private Nat state followed by
Unit. Other directions, bodies, context observation, multiple yields,
post-resumption state access or mutation, effectful traversal actions,
abandonment, close delivery, overlap, and termination interaction remain
rejected with stable diagnostics.

Stream construction shall capture the task capability without loading state.
Its single traversal shall load state at the source yield after the preceding
event commit, perform one inert Unit resumption, commit a successful Unit final
Result, and complete before the following request begins. Since no admitted
code can observe the resume carrier, retain a continuation, or access task
state after resumption, generated O0 code may inline the suspension and erase
those carriers after the checked model records the complete stream directions
and transaction. This semantic lowering shall not depend on LLVM optimization.

The Linux x86-64 backend shall reuse the private task, exact Nat, and Result
runtime representations and add no foreign dependency, C/C++ runtime,
other-language standard library, public ABI, or native ABI revision. Task and
Generator DWARF plus the bundled GDB renderers shall expose the owning task,
private state, affine stream directions, stream binding, and source yield.
Functional evidence shall include checked-model metadata and rejection tests,
ordered O0 LLVM, exact interpreter differential, complete compiler corpus,
freestanding ELF/DWARF/GDB inspection, and separate build/run resource
baselines. Future compiled-library metadata shall carry stream directions,
transaction/suspension identity, state-authority release and reacquisition,
ownership/consumption, close/termination behavior, effects, authorities, debug
provenance, and native adapter requirements independently of private LLVM
types, layouts, and symbols. This realizes
`TOPAL-COMPILER-TASK-STREAM-001`, `TOPAL-TASK-HANDLER-001`, and
`TOPAL-TASK-MESSAGE-001` for increment 7b3.

## TOPAL-COMP-EXTERNAL-LOCATION-001 — Closed checked external location

The checked compiler shall admit the unchanged external-layout-location
regression as compiler increment 7b4. It shall retain target-independent
metadata for the complete five-layout graph, including identities, semantic
classifiers, sizes, encodings, endianness, access, alignment, product fields and
packing, tagged-sum tags and placement, array extent, element layout and stride.
It shall likewise retain the address-range identity, policies, medium, minimum
access size and inclusive bounds; the offset subtype's exact range identity,
alignment and checked value; and the Location subtype, layout and offset.

The checker shall reject missing, extra, duplicate, inconsistent, dynamic, or
unsupported fields; unrepresentable UInt32LE values; invalid bounds, offsets,
fit, alignment, or access; and every non-closed location flow. The admitted
write and read shall be distinct, source ordered, and exactly differential with
the interpreter. General encoders/decoders, fallible hardware access,
uninitialized reads, other layout families, and function, persistent, public,
serialized, or compiled-library location boundaries remain unsupported.

Because the source grants no Linux device mapping or native-adapter authority,
the backend shall not dereference the numeric MMIO address. It shall allocate a
private process-owned Topal Location header through the existing Linux syscall
allocator and preserve range start, offset, initialization state, and the
immutable semantic Nat snapshot. Noinline Topal runtime write/read calls shall
preserve O0 ordering. This explicitly models only the authority-free regression
and does not claim real MMIO integration; that requires a future typed platform
adapter. The executable shall add no C/C++ runtime, other-language standard
library, foreign allocator, undefined symbol, dynamic dependency, public ABI,
or native ABI revision.

DWARF and the bounded validating GDB renderer shall expose the nominal
UInt32LE-backed Nat, nominal ControlLocation, semantic address evidence,
initialization state, stored value, source operations, and frames. Functional
evidence shall cover metadata retention and rejection, ordered O0 LLVM, exact
interpreter differential, freestanding ELF and DWARF/GDB inspection, the full
compiler corpus, and separate build/run resource baselines. Future library
metadata shall carry the complete language/schema/layout/range/offset/location,
ordering, effect, authority, fallibility, ownership, platform-adapter, and debug
contracts independently of LLVM types, layouts, headers, and symbols. This
realizes `TOPAL-COMPILER-EXTERNAL-LOCATION-001`, `TOPAL-LAYOUT-SIZE-001`,
`TOPAL-LAYOUT-CONSTRUCT-001`, `TOPAL-ADDRESS-RANGE-001`,
`TOPAL-LOCATION-CONSTRUCT-001`, `TOPAL-LOCATION-READ-001`, and
`TOPAL-LOCATION-WRITE-001` for increment 7b4.

## TOPAL-COMP-DEBUG-001 — DWARF and GDB

Debug-enabled O0 output shall map generated source functions, parameters,
immutable scalar locals, lexical scopes, and instructions to Topal files and
source locations, emit DWARF 5 through LLVM, retain frame pointers, provide GDB
renderers for private arbitrary-precision Int, Rational, `Range Int`,
`Range Rational`, `Optional Int`, `Optional String`, `Optional List Int`,
`Optional (Int, List Int)`, `Optional Error`,
`Optional SourceLocation`, `SourceLocation`, nominal modular-number objects,
modular-success Result objects, `List Effect`, `List Int`,
`List (String, Int)`, exact-extent `Array Int`, `Set Int`, `Bag Int`, and
`Map (String, Int)` container values, describe source-declared nominal enums,
native `SerializationStream` descriptors,
nominal direct Task instances with identity, lifecycle state, and private Nat
state, affine task-stream Generator directions and bindings,
nominal layout-backed Nat and checked Location headers with address evidence,
initialization state, and stored snapshot,
retained Constraint identities and refined Int bindings with their semantic
names, distinguish
selected overload and static-function frames, and pass automated GDB
breakpoint, value, and backtrace scenarios.

## TOPAL-COMP-ENUM-001 — Payload-free nominal enums

The compiler shall recognize admitted root-scope payload-free
`Name is Enum (A, …)` declarations as distinct from general `Union`, preserve
their declaration-scoped nominal identity, classify their alternatives and
scalar function boundaries, and permit equality only within the same enum type.
Each declaration shall use a declaration-ordered `i32` tag in the sealed private
ABI; the representation shall not be exposed as a portable foreign enum
convention.

Enum display shall emit the selected source label through the Topal platform
writer. Enum decisions shall evaluate their subject once, select only the
first matching alternative action, diagnose missing, foreign, or unknown
alternatives, and merge compatible reachable scalar actions through LLVM
control flow. Generated invalid-tag paths shall fail closed as compiler-runtime
corruption. DWARF shall retain the nominal type and every enumerator so GDB
renders parameters and locals as source alternatives without a language-runtime
dependency.

This requirement covers `TOPAL-TYPE-ENUM-001` and the source-declared enum case
of `TOPAL-DECISION-ENUM-001`. It realizes `TOPAL-COMPILER-ENUM-001` for
compiler increment 3b1; nested enum declarations and general unions remain in
increment 3b2.

## TOPAL-COMP-SUM-001 — Private nominal Union and Variant values

The checked compiler shall collect root-scope `Name is Union` declarations and
`Name is Variant (...)` bindings, retain one nominal identity and the ordered
payload classifier of every alternative, and admit payload types that already
have recursively complete private representations. Payload-free Union
alternatives shall carry Unit without a payload slot. Construction shall
evaluate and classify the selected complete payload exactly once. Complete
decisions shall evaluate their subject once, select by the retained tag, bind a
payload only in its selected action, and diagnose unknown, foreign, duplicate
declaration, or missing alternatives at the checked boundary.

The Linux x86-64 backend shall use one non-packed LLVM literal struct containing
a declaration-ordered `i32` tag followed by one statically typed slot for every
payload-bearing alternative. It shall initialize all machine slots, observe
only the active one, and use the same exact type in private `fastcc`
definitions, calls, and returns. LLVM shall own physical register/stack
classification. Invalid tags shall take the compiler-runtime corruption exit.
Source display shall match the interpreter's labeled or positional alternative
spelling through the Topal writer.

DWARF shall expose the nominal source typedef, an enumerated source-alternative
tag, and correctly laid-out payload members. Named values and aggregate payload
bindings shall use target-aligned debug-only shadows where LLVM 22 cannot retain
an inspectable SSA aggregate. The bundled GDB renderer shall show only the
active alternative and payload. This shall add no semantic heap allocation,
sum runtime helper, foreign dependency, C/C++ runtime, other-language standard
library, public aggregate ABI, or `topal-native/6` revision. Recursive nominal
sums, nested declarations, derived ordering, persistent/public
storage, serialization, introspection, and library metadata remain deferred.
This realizes `TOPAL-COMPILER-SUM-001`, `TOPAL-TYPE-UNION-001`,
`TOPAL-TYPE-VARIANT-001`, and `TOPAL-DECISION-UNION-001` for compiler increment
3b2-b5o.

## TOPAL-COMP-SUM-EQUALITY-001 — Derived nominal Sum equality

The checked compiler model shall admit `=` and `!=` for two values of one exact
nominal Union or positional Variant type exactly when every declared payload
has admitted canonical Equality. Payload-free alternatives shall contribute
Unit. Tuple and Record payloads and fields may recursively contain an admitted
Sum. Function, Range, Result, authority-bearing, and other payloads without
admitted Equality shall reject the complete Sum operation before LLVM lowering
or artifact publication. Structurally identical distinct declarations shall
not share an overload.

Lowering shall compare tags first, return false for different tags without
observing either payload, and use LLVM `switch` to select and recursively
compare only the active payload for equal tags. Payload-free alternatives shall
return true, invalid private tags shall fail closed as unequal, and an `i1`
`phi` shall join the explicit predecessor results at O0. `!=` shall negate the
same equality result. The operation shall reuse the existing exact private
tag-plus-payload aggregate, matching `fastcc` prototypes, nominal DWARF type,
and active-value GDB rendering while LLVM owns AMD64 physical lowering.

Tests shall cover labeled Union and positional Variant values; payload-free,
Int, String, Tuple, Record, and nested Sum alternatives; equal payloads;
same-tag payload inequality; distinct-tag short-circuiting; nominal mismatch;
unsupported-payload rejection before publication; direct `switch`/`phi` IR;
interpreter parity and reversible history; the shared corpus and separate
resource baselines; freestanding ELF/DWARF properties; and full O0 GDB
values/frames. This shall add no whole-storage comparison, inactive-field load,
generic matcher, Sum-equality runtime, allocation, callback, indirect dispatch,
foreign dependency, C/C++ runtime, other-language standard library, public Sum
ABI, or `topal-native/6` revision. Future compiled-library metadata shall encode
nominal identity, ordered alternatives, payload schemas, canonical Equality
evidence, representation identity, and target adapters independently of private
tags, inactive storage, LLVM types, and physical placement. Recursive Sum
declarations, persistent/public storage, publication, and library adapters
remain deferred. This realizes `TOPAL-COMPILER-SUM-EQUALITY-001`,
`TOPAL-TYPE-SUM-EQUALITY-001`, and `TOPAL-TYPE-EQUALITY-001` for compiler
increment 3b2-b5ah.

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
