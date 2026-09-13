# Native compiler and artifact conformance

## Formal text

### TOPAL-COMPILER-TARGET-001 — Qualified initial target

Compiler target identity SHALL include architecture, vendor, operating system,
environment, data layout, object format, CPU baseline, and features. Compiler
revision 1 SHALL accept only `x86_64-unknown-linux-gnu` with the qualified
layout and x86-64 baseline recorded in `se/compiler-architecture.md`; it SHALL
reject every other target before lowering. Host CPU discovery SHALL NOT silently
strengthen the emitted feature set.

### TOPAL-COMPILER-PLATFORM-001 — Freestanding Linux execution

The initial executable SHALL define its own process entry and SHALL link with
no foreign-language startup object, standard library, runtime, default library,
or ELF interpreter. Linux interactions SHALL occur only through target support
operations whose exact syscall number, register convention, memory effects,
partial completion, interruption, and error behavior are explicit. The first
support operations SHALL implement complete standard-output writes, private
anonymous mappings for compiler-owned runtime storage, and process termination.

### TOPAL-COMPILER-O0-001 — Unoptimized semantic reference

`-O0` SHALL perform every mandatory parse, type, evidence, representation,
cleanup, and target-validity lowering, but SHALL perform no optional
source-semantic rewrite or specialization beyond that required to represent an
accepted program. LLVM verification and correctness-preserving backend lowering
remain mandatory. The resulting executable's observable value and trace SHALL
equal the interpreter's for every source in their shared implemented subset.

### TOPAL-COMPILER-INT-001 — Arbitrary finite Int representation

Every admitted finite `Int` SHALL be represented without a fixed machine-word
bound. Its private native object SHALL use one canonical zero encoding, a
normalized sign, no redundant high limbs, and enough dynamically allocated
limbs to hold the exact magnitude. Integer negation, absolute value, addition,
subtraction, multiplication, equality, ordering, function passage,
control-flow joins, and decimal observation SHALL preserve the corresponding
`TOPAL-NUM-*` value for all operand sizes that available target storage can
hold.

Runtime operations SHALL NOT call an undeclared foreign allocator, arithmetic
helper, unwinder, or standard library. Storage failure SHALL follow an explicit
platform-failure path rather than producing a truncated or invalid `Int`.

### TOPAL-COMPILER-EXACT-001 — Finite exact-number runtime

Every admitted finite `Rational` SHALL retain canonical coprime arbitrary-
precision `Int` numerator and positive denominator values, including canonical
zero. Closed construction, exact decimal literals, Int embedding and division,
Rational negation, absolute value, addition, subtraction, multiplication,
division, natural and negative powers, equality, ordering, three-way
comparison, decimal observation, function passage, and control-flow joins
SHALL preserve the corresponding `TOPAL-NUM-*` value without approximation.

Finite Int power, Euclidean modulo, and quotient/modulo SHALL use the same
unbounded representation and normative sign rules. A statically evident zero
divisor SHALL remain a source diagnostic. Every admitted dynamic failure path
SHALL follow `TOPAL-COMPILER-RESULT-001`; a compiler SHALL NOT replace a typed
Result with process termination.

Position-independent output without an ELF interpreter SHALL contain no
load-time pointer relocation. Runtime construction SHALL be used when a private
aggregate would otherwise require such a relocation.

### TOPAL-COMPILER-NAT-COMPARISON-001 — Nat comparison evidence

For equality, inequality, ordered predicates, and three-way comparison, an
admitted Nat operand SHALL forget its validated constraint evidence to its
unchanged exact Int base value. Nat/Int comparison SHALL use canonical Int
comparison. Nat/Rational comparison SHALL apply the same canonical
Int-to-Rational conversion as the underlying Int value. Each source operand
SHALL be evaluated exactly once before evidence forgetting or conversion.

Evidence forgetting SHALL NOT narrow, copy, reinterpret as an unsigned machine
integer, allocate another Int object, call a Nat-specific runtime operation, or
change the Nat identity of a source binding in debug information. An admitted
same-classifier positional product MAY recursively use this Nat equality as
field evidence under `TOPAL-COMPILER-TUPLE-EQUALITY-001`.

### TOPAL-COMPILER-RANGE-001 — Finite exact ranges

Every admitted finite explicitly bounded `Range Int` and `Range Rational`
SHALL retain its exact lower and upper endpoints and both inclusivity states.
Construction, mixed Int-to-Rational endpoint conversion, classification,
function passage, control-flow joins, membership by either operand order,
intersection, emptiness, bound observation, inclusivity observation, textual
output, and debugging SHALL preserve the corresponding `TOPAL-RANGE-*`
semantics without enumeration or endpoint adjustment.

The private representation SHALL remain opaque outside its versioned native
ABI and SHALL contain no load-time pointer relocation in output without an ELF
interpreter. Unbounded ranges, infinite endpoints, and range-based collection
selection SHALL remain outside the admitted subset until their prerequisite
semantics and value representations are implemented.

### TOPAL-COMPILER-DECISION-001 — Comparison decision control flow

For every admitted comparison-matcher decision, generated control flow SHALL
evaluate the subject exactly once, evaluate matcher operands in source order
only when their rule is reached, execute only the first selected action, and
use the required final `otherwise` action when no comparison succeeds. Mixed
exact operands SHALL retain the same canonical conversion and comparison
semantics as an ordinary application.

For every admitted decision over a `Comparison` value, generated control flow
SHALL distinguish the closed `Less`, `Equal`, and `Greater` alternatives and
execute exactly the selected action. Both forms SHALL preserve compatible
machine-scalar action values across the merge without introducing eager source
evaluation or a runtime-library dispatch dependency.

### TOPAL-COMPILER-RESULT-001 — Dynamic arithmetic Result representation

Every admitted arithmetic `Result` SHALL preserve a success-or-error tag and a
payload classified by the declared success type or the common structured Error
type. An intrinsic Error SHALL retain its compiler-derived reporting domain,
nominal arithmetic code, absent detail and cause, and source file, line, and
column provenance. Passing or returning a Result SHALL preserve those fields
unchanged, while an ordinary successful value SHALL satisfy its explicit Result
contract without a source-level wrapper operation.

Dynamic finite Rational construction, Rational division and negative power,
and Int modulo and quotient/modulo SHALL construct their specified arithmetic
Error instead of terminating or exposing an undefined runtime operation. The
private representation and function signatures SHALL remain sealed by the
exact native-ABI revision, shall be debuggable at O0, and shall introduce no
foreign allocator, runtime, or calling convention.

Exact checked Int and Nat construction and a classified Rational-to-Int
binding SHALL preserve proven values directly and use the same structured
Result representation for dynamic validation. Contextual success projection
SHALL return the original complete Error from a compatible enclosing Result
function before evaluating later statements; only the success continuation may
load and reclassify the payload. Validation SHALL neither round, truncate,
clamp, nor reconstruct a propagated Error.

Every admitted Result decision SHALL evaluate its subject once, make `Ok` and
whole-`Error` bindings available only to their selected actions, and evaluate
only the selected action. A qualified arithmetic Error-code matcher SHALL test
the stored nominal code independently of `Error.domain`. One `Ok` case plus a
whole-Error fallback or every member of the closed arithmetic code vocabulary
SHALL be exhaustive; unknown, duplicate, and unreachable code cases SHALL be
diagnosed.

Selecting the admitted `code` and `domain` fields SHALL return their stored
typed values without rebuilding the Error. String-valued decision actions,
function boundaries, display, debug information, and Result payloads SHALL use
one immutable native String descriptor. A relocation-free executable SHALL
construct pointer-bearing descriptors at run time and print canonical ordinary
or conflict-free tagged Topal literal syntax without a foreign runtime.

### TOPAL-COMPILER-ERROR-CODE-001 — Qualified arithmetic code identity

Each qualified value in the closed `lang arithmetic ArithmeticErrorCode`
vocabulary SHALL use the same nominal identity, private tag, display label, and
DWARF enumerator whether constructed directly or observed from an Error. Direct
values SHALL support same-type equality and admitted scalar function passage.
Generated code SHALL NOT perform a runtime namespace lookup or introduce a
foreign runtime dependency for a statically qualified code.

### TOPAL-COMPILER-FUNCTION-001 — Selected scalar function identities

Within the admitted scalar-function subset, the compiler SHALL preserve
source-ordered overload sets and reject a repeated input-classifier sequence
with the same staticness independently of parameter names and result type. A
call SHALL evaluate its argument expressions once, select the first statically
applicable complete input header without consulting result context, and emit a
distinct checked call-graph identity and private native function for the
selected overload.

Complete explicitly classified function headers in one declaration scope SHALL
be collected before any selected body is checked. An admitted acyclic body MAY
therefore call a function declared later in that scope. The callee SHALL still
be checked and emitted before its caller, and ordinary value initializers SHALL
remain source-ordered rather than acquire forward visibility.

Static nullary, unary, and positional-product functions SHALL use the same
private scalar representations as ordinary functions. A static function body
SHALL NOT select an ordinary callee. LLVM lowering and DWARF SHALL retain each
selected overload's source function, typed parameters, invocation-local
bindings, and frame without exposing staticness as an unqualified foreign ABI.

### TOPAL-COMPILER-RECURSION-INT-001 — Proven direct Int recursion

The compiler SHALL admit a direct unary decreasing `Int` recursion edge only
when the shared language proof establishes
`TOPAL-FUNCTION-RECURSION-INT-001`, including the strict literal-step rule of
`TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001` for every self-call. The base
action, zero or invalid steps, and indirect cycles without an implemented proof
SHALL remain rejected. Static facts from the initial call argument SHALL NOT be
assumed for the recursive function parameter; checking the body SHALL use the
declared `Int` classifier conservatively across every invocation.

One proven recursive overload SHALL lower to one compiler-private function and
one exact LLVM prototype. Every self-call SHALL use the same calling convention
and symbol. Correctness at O0 SHALL NOT depend on inlining, tail-call conversion,
or any other LLVM optimization, and the function SHALL NOT be marked
`norecurse`. DWARF SHALL retain each non-inlined recursive frame and its current
source parameter. This admission SHALL add no runtime dispatch, foreign
dependency, standard library, or native ABI revision.

### TOPAL-COMPILER-RECURSION-INT-INCREASING-001 — Proven direct increasing Int recursion

The compiler SHALL admit the increasing dual of
`TOPAL-COMPILER-RECURSION-INT-001` only when the shared language proof
establishes `TOPAL-FUNCTION-RECURSION-INT-INCREASING-001`. The complete body
SHALL use the inclusive upper-bound form, its base SHALL contain no self-call,
and every recursive edge SHALL add a strict positive literal step satisfying
`TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001`. Zero, negative,
runtime-selected, subtracting, and otherwise unproven steps SHALL remain
rejected.

The same conservative parameter checking, one-symbol exact private prototype,
O0 correctness, LLVM attribute, DWARF frame, freestanding-runtime, and native
ABI obligations of `TOPAL-COMPILER-RECURSION-INT-001` SHALL apply. Positive
multi-unit steps MAY overshoot the inclusive bound as specified by the shared
proof and SHALL NOT require a distinct runtime operation or representation.

### TOPAL-COMPILER-RECURSION-NAT-001 — Proven direct Nat recursion

The compiler SHALL admit direct unary `Nat` recursion only when the shared
language proof establishes `TOPAL-FUNCTION-RECURSION-NAT-001` or
`TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001`. A decreasing edge SHALL satisfy
the proof's nonnegative inclusive-bound and maximum-step obligations; an
increasing edge SHALL add a strict positive literal. Unsafe overshoot, a
negative bound for decreasing recursion, a zero or wrong-direction step, a
self-call in the base, and otherwise unproven edges SHALL remain rejected.

Checking a recursive step SHALL first forget `Nat` constraint evidence to the
unchanged exact `Int` value for addition or subtraction. The compiler MAY
reattach `Nat` evidence to the recursive argument without dynamic validation
only after the applicable shared proof establishes that every such argument is
nonnegative. This evidence operation SHALL emit no machine instruction,
validation call, unsigned conversion, allocation, or representation change.
The function SHALL otherwise meet the exact private prototype, O0, attribute,
DWARF, freestanding-runtime, and native ABI obligations of
`TOPAL-COMPILER-RECURSION-INT-001`.

### TOPAL-COMPILER-FUNCTION-DECREASES-001 — Explicit measured recursion

The compiler SHALL admit an explicitly measured direct recursive function only
when the shared language proof establishes `TOPAL-FUNCTION-DECREASES-001` for
the complete selected overload. The initial compiler subset SHALL accept one
directly named `Int` or `Nat` measure across a scalar multi-parameter state and
SHALL verify every recursive packaged argument. A missing or mismatched measure,
an invalid decision shape, or any non-progressing edge SHALL remain rejected;
the written `Decreases` clause alone SHALL NOT be trusted.

Proof metadata SHALL identify the measured parameter exactly. Only that
position MAY regain `Nat` evidence without dynamic validation when the shared
proof also preserves its domain; no other parameter receives that authority.
The explicit measure SHALL add no hidden machine parameter, runtime counter,
allocation, validation call, foreign dependency, standard library, or native
ABI revision. One recursive overload SHALL retain one exact private prototype,
and DWARF/GDB SHALL expose all source parameters in each non-inlined frame.

### TOPAL-COMPILER-ENUM-001 — Sealed nominal enum lowering

Each admitted payload-free source enum SHALL retain a distinct nominal identity
through checking and use declaration-ordered `i32` tags only within the sealed
Topal native ABI. Classification, equality, function passage, control-flow
joins, and display SHALL preserve `TOPAL-TYPE-ENUM-001`; no tag SHALL be
implicitly exchanged with another enum or treated as a portable foreign enum.

An admitted enum decision SHALL evaluate its subject once, execute only the
selected alternative or final `otherwise` action, and enforce the completeness
and matcher-validity rules of `TOPAL-DECISION-ENUM-001`. LLVM `switch` and
`phi` lowering SHALL retain those evaluation semantics. An invalid internal tag
SHALL fail closed rather than select a source alternative. DWARF SHALL describe
the nominal enum and its source labels truthfully for GDB inspection.

### TOPAL-COMPILER-RETURN-001 — Mandatory direct-return lowering

For every admitted direct `return` in a linear function body, the checked
compiler model SHALL evaluate and validate the return expression once, retain
all preceding statement effects in source order, and exclude every later
statement in that invocation from LLVM IR. This exclusion is mandatory
semantic lowering at `-O0`, not dead-code optimization. The backend SHALL use
the function's ordinary private result representation and truthful return-line
debug location. A root-level return SHALL be rejected.

### TOPAL-COMPILER-BLOCK-001 — Lexically scoped block lowering

For the admitted cleanup-free subset of `TOPAL-EXEC-BLOCK-001`, an empty block
SHALL produce Unit without allocation. An admitted
nonempty block SHALL evaluate statements in source order in a fresh child
binding environment and deliver its final value. A child binding MAY shadow an
outer name, SHALL become visible only after its initializer, and SHALL NOT
escape the block. Lowering SHALL preserve those rules without mutating the
enclosing compiler or generated-value environment.

Every generated instruction and machine-represented immutable local belonging
to the block SHALL carry a nested DWARF lexical scope. A source block form whose
exit or lifetime semantics have no admitted lowering SHALL be rejected rather
than flattened into the enclosing scope.

### TOPAL-COMPILER-COMPLETED-001 — Retained completion evidence

Completed SHALL remain a distinct checked and generated value type from Unit.
An admitted function returning Completed SHALL use a typed zero-data private
result and its call SHALL remain an ordering dependency at O0; it SHALL NOT be
lowered as a `void` Unit call. Construction SHALL require no allocation.

Same-type equality, scalar function passage, control-flow joins, canonical
display, DWARF, and GDB SHALL preserve the singleton source identity. Its
machine carrier SHALL NOT be exposed as a public integer or foreign ABI.

### TOPAL-COMPILER-OPTIONAL-001 — Native Optional values

Within the admitted `Int` and `String` payload subset, `Optional T` SHALL retain
its nominal payload classifier and a distinct present-or-absent alternative at
construction, classified bindings, ordinary function parameters and results,
direct returns, control-flow joins, display, DWARF, and GDB inspection. Bare
`None` SHALL be admitted only where the immediate classified binding or
function-result context determines `Optional T`; explicit `None T` and
`Some value` SHALL preserve the same identity rules.

An admitted Optional decision SHALL evaluate its subject once, bind the
present payload only in the selected `Some` action, execute exactly one action,
and require both alternatives or `otherwise`. Derived equality SHALL be
available only for an admitted payload type whose canonical equality is
implemented; it SHALL compare payloads only when both alternatives are `Some`,
make two `None` alternatives equal, and make unlike alternatives unequal.

The native Optional representation SHALL remain distinct from Result even if
their private storage shapes coincide. Its tag SHALL be validated before an
alternative-sensitive observation, and its opaque payload pointer SHALL be
loaded only after `Some` is established. The header SHALL be constructed
without load-time pointer relocations, passed only through sealed Topal
signatures, and rendered without a foreign allocator, runtime, or standard
library.

### TOPAL-COMPILER-OPTIONAL-RATIONAL-001 — Exact Optional Rational values

The native Optional rules SHALL extend to `Optional Rational` without changing
the Optional header, Rational payload, Topal-private function signature, or
native ABI revision. Construction, classified and contextual absence,
function passage and result, direct return, control-flow joins, decisions,
display, DWARF, and GDB SHALL retain the Rational payload classifier.

Derived equality SHALL validate both Optional alternatives and invoke exact
canonical Rational equality only when both values are `Some`. Two `None`
values SHALL compare equal, and unlike alternatives SHALL compare unequal
without loading an absent payload. The resulting equality SHALL compose as an
admitted positional-product field equality under
`TOPAL-COMPILER-TUPLE-EQUALITY-001`.

### TOPAL-COMPILER-TUPLE-EQUALITY-001 — Derived positional-product equality

An admitted equality or inequality between positional products with the same
field classifiers SHALL evaluate the complete left operand and then the
complete right operand exactly once. It SHALL recursively apply each field's
admitted canonical equality and make the products equal exactly when every
corresponding field is equal. Inequality SHALL be the Boolean negation of that
same result.

Lowering MAY retain a product as an internal aggregate of field values when it
does not cross a machine boundary. It SHALL NOT allocate storage or expose a
public aggregate ABI solely to compare the product. Product equality requiring
a canonical conversion between differently classified corresponding fields
SHALL remain unsupported until that conversion has an admitted product
lowering; the compiler SHALL reject it at the checked boundary rather than
silently compare incompatible representations.

### TOPAL-COMPILER-RECORD-001 — Anonymous record construction and selection

An admitted anonymous product whose fields are all labeled SHALL construct a
structural Record. Field expressions SHALL be evaluated once from left to right,
labels SHALL be unique, and canonical display SHALL retain construction order.
The inferred type SHALL retain every label and exact field classifier while
treating the label set canonically for static identity. Selecting `record label`
SHALL return that field's already-evaluated value with its exact classifier; an
absent label SHALL be rejected at the label source range.

The admitted expression-local Record MAY remain a decomposed compiler aggregate
and SHALL require no allocation, native object header, generated runtime, or
foreign aggregate ABI. Record passage through machine signatures and aggregate
storage remain unsupported. Until that storage exists, the compiler SHALL
expose source locations and projected scalar bindings through DWARF/GDB but
SHALL NOT publish a misleading aggregate-local debug representation.

### TOPAL-COMPILER-STRUCTURAL-COMPARISON-001 — Derived structural comparison

An admitted Tuple SHALL provide total ordering when every corresponding field
has an admitted total order, including recursively nested Tuples. Both complete
operands SHALL evaluate exactly once from left to right. Field comparison SHALL
then proceed lexicographically in position order and SHALL stop at the first
non-Equal result. `<`, `>`, `<=`, `>=`, and `<=>` SHALL select their result from
that single ordering. Canonical Int/Nat/Rational conversion SHALL be applied per
field before comparison when required.

An admitted anonymous Record SHALL provide equality and inequality when both
operands have the same canonical label set and every corresponding field has an
admitted equality. Fields SHALL align by label regardless of construction order,
and canonical field conversions SHALL occur before equality. A different label
set or a field pair with no common evidence SHALL have no applicable structural
comparison rather than compare unequal.

Lowering SHALL operate on the existing decomposed aggregates, use short-circuit
LLVM control flow for lexicographic comparison, and introduce no aggregate
allocation, runtime, or native ABI. A required field conversion hidden inside
an opaque aggregate SHALL remain unsupported until aggregate projection or
storage exists.

### TOPAL-COMPILER-RECONSTRUCT-001 — Immutable record reconstruction

For an admitted Record `base`, `base with (field is replacement, ...)` SHALL
evaluate `base` exactly once before evaluating each replacement exactly once in
source order. Every replacement label SHALL occur exactly once, name a field of
`base`, and satisfy that field's exact classifier through an admitted canonical
conversion when required. The result SHALL retain the base Record's classifier
and display order, retain every unreplaced field value, and leave `base`
unchanged.

The checked representation MAY reconstruct an expression-local decomposed
Record by replacing its named compiler values. This lowering SHALL introduce no
aggregate allocation, generated reconstruction runtime, public ABI, foreign
runtime, or standard-library dependency. Projected scalar values SHALL retain
the existing DWARF/GDB behavior; the decomposed aggregate SHALL remain absent
from debugger locals until truthful Record storage and layout exist.

### TOPAL-COMPILER-STRING-UTF8-BYTE-COUNT-001 — Native prospective byte count

For an admitted plain String, `text byte-count Utf8` SHALL read the exact
preserved UTF-8 byte length from the immutable native String descriptor and
produce the equal nonnegative value in the canonical arbitrary-precision Int
representation. It SHALL evaluate `text` once and SHALL NOT inspect the cached
display spelling, mutate or normalize the String, attach an encoding, count
characters or display columns, or narrow the result to a source-level machine
integer.

Converting the target descriptor length into Int SHALL use only the sealed
Topal runtime representation and platform allocator. It SHALL NOT call a
foreign String, encoding, conversion, allocator, or standard-library routine.

### TOPAL-COMPILER-STRING-EQUALITY-001 — Exact native String equality

An admitted same-type String equality or inequality SHALL compare the complete
preserved Unicode sequence exactly, including distinctions between canonically
equivalent but differently represented sequences. With the qualified valid
UTF-8 descriptor representation, lowering SHALL compare preserved-byte lengths
and then corresponding bytes in order. It SHALL NOT compare cached display
spellings, normalize either value, consult locale state, or call a foreign
string routine.

Derived `Optional String` equality SHALL validate each Optional alternative,
compare payload Strings only when both alternatives are present, make two
absent alternatives equal, and make different alternatives unequal without
loading an absent payload. Equality and inequality SHALL evaluate each source
operand once and preserve the result required by `TOPAL-TYPE-EQUALITY-001` and
`TOPAL-TYPE-OPTIONAL-EQUALITY-001` at `-O0`.

### TOPAL-COMPILER-STRING-CONSTRUCTION-001 — Native String construction

`empty String` SHALL construct the zero-length plain String value, and adjacent
String literal primaries SHALL compose their preserved sequences in source
order as mandatory construction semantics. An admitted plain `concat` SHALL
evaluate both operands exactly once from left to right, fail through the
explicit platform storage path when their combined target length is not
representable, allocate through the Topal platform boundary, and copy the
complete preserved UTF-8 byte sequences in order. None of these paths SHALL
normalize, reinterpret, or add content.

`empty?` SHALL evaluate its String operand once and return true exactly when its
preserved sequence is empty. A dynamically constructed String whose descriptor
does not cache display spelling SHALL still render the canonical ordinary or
shortest collision-free tagged Topal literal directly from its preserved bytes.
Generated execution and GDB SHALL expose the same valid String value without a
foreign allocator, runtime, standard library, locale, or text transformation.

### TOPAL-COMPILER-CHARACTER-001 — Retained static Character evidence

For a closed String expression whose preserved sequence is known during
checking, the compiler SHALL classify it as Character exactly when the pinned
language-context segmentation counts one extended grapheme cluster. It SHALL
diagnose a closed zero- or multiple-character expression and SHALL reject
dynamic Character validation until its explicit Result and runtime Unicode
path are admitted; it SHALL NOT substitute byte or Unicode-scalar count.

An admitted Character SHALL retain its classifier through classified bindings,
ordinary function parameters and results, direct returns, equality, product
fields, DWARF, and GDB. Forgetting Character evidence through `String value`
or an implicit base conversion SHALL preserve the exact sequence and require no
generated instruction. The machine value SHALL remain the same immutable
String descriptor, exact equality SHALL reuse canonical String equality, and
no native ABI revision or foreign Unicode/runtime dependency SHALL result.

### TOPAL-COMPILER-CHARACTER-OBSERVATION-001 — Closed Character observations

When the complete preserved String and, for indexing, exact Int index are known
during checking, `character-count`, String `entry-count`, and `character-at`
SHALL be evaluated under the selected language context's pinned extended-
grapheme segmentation. Both counts SHALL produce the equal arbitrary-precision
Int. Indexing SHALL produce `Some Character` containing the complete preserved
cluster or `None` for a negative or out-of-range index.

This evaluation SHALL be mandatory at `-O0`, not delegated to an LLVM
optimization. The admitted `Optional Character` result SHALL use the existing
private Optional header and String-descriptor payload through function passage,
decisions, display, DWARF, and GDB. Dynamic text or index observations SHALL be
rejected until a Topal-owned freestanding Unicode runtime path is admitted;
the compiler SHALL NOT substitute bytes, scalar values, host Unicode tables,
locale services, or another language's runtime.

### TOPAL-COMPILER-UNICODE-FOLD-001 — Closed pinned-Unicode operations

For a closed String expression whose complete preserved sequence is known
during checking, `upper`, `lower`, full `case-fold`, NFC normalization, and NFD
normalization SHALL be evaluated with the selected language context's pinned
Unicode data. Each result SHALL be a plain immutable String containing exactly
the normative transformed scalar sequence; the input value SHALL remain
unchanged. Closed `canonically-equals` SHALL evaluate canonical equivalence
without changing either operand and SHALL return the required Boolean.

These evaluations SHALL be mandatory at `-O0`, not delegated to an LLVM
optimization. Their generated values SHALL use the existing String descriptor,
display, DWARF, and GDB paths. Unknown operands SHALL be rejected until a
Topal-owned freestanding Unicode runtime is admitted; generated code SHALL NOT
consult host Unicode tables, locale services, C/C++ runtimes, or standard
libraries, and the native ABI SHALL NOT change for these operations.

### TOPAL-COMPILER-PATTERN-001 — Discarded machine inputs

An admitted positional-product prefix application SHALL evaluate and validate
its fields once in source order before passing them in declared parameter
order. A typed discard parameter SHALL participate in overload matching and
occupy its target-qualified private machine-signature position, but SHALL
introduce no source binding, generated-value binding, or DWARF variable.

Retained parameters after a discard SHALL keep their original source argument
ordinals. Lowering SHALL NOT expose the discarded value under a synthetic or
otherwise addressable Topal name.

### TOPAL-COMPILER-LLVM-001 — LLVM module and tool qualification

Every LLVM module SHALL carry the exact qualified target triple and data layout,
use only supported standard LLVM semantics, and pass LLVM verification before
object emission. The compiler SHALL reject an incompatible LLVM major. The
invoked LLVM version, CPU baseline, optimization level, relocation model, and
debug mode SHALL be reproducible artifact inputs.

### TOPAL-COMPILER-ABI-001 — Sealed native ABI

Every private machine signature SHALL belong to an exact Topal native-ABI,
compiler, language, and target revision. It SHALL NOT be treated as a public
foreign ABI. A future foreign adapter SHALL use the target's qualified calling
convention and SHALL expose only explicitly described fixed-width scalars,
opaque handles, or independently verified aggregate coercions.

### TOPAL-COMPILER-ARTIFACT-001 — Complete native manifest

A native artifact manifest SHALL identify its schema, language and GEIR
revisions, target and data layout, object and platform ABIs, CPU/features,
compiler and LLVM versions, optimization/debug modes, source/interface and
dependency digests, exports, platform requirements, evidence, provenance, and
native slices. A consumer SHALL validate all required fields and digests before
using machine code or semantic metadata. LLVM IR and bitcode SHALL be
LLVM-major-qualified rebuildable payloads rather than the compatibility
boundary.

### TOPAL-COMPILER-DEBUG-001 — GDB-observable unoptimized code

Debug-enabled `-O0` output SHALL emit DWARF source files, line mappings,
subprograms, machine-represented parameters and immutable locals, their
supported value types, and unwindable frames sufficient for tested GDB source
breakpoints, stepping, backtraces, and value inspection. Compiler-generated
platform frames SHALL be distinguishable from source functions. A source value
without a truthful machine representation SHALL be omitted until that
representation is implemented; it SHALL NOT be represented by an unrelated
machine value. A debugger renderer MAY decode a documented private object into
its source value, but SHALL reject corrupt representation metadata safely.

### TOPAL-COMPILER-TEST-001 — Shared executable regressions

Compiler conformance tests SHALL consume the same Topal source identity as the
interpreter test, compile it at `-O0`, execute the resulting binary, and compare
its observable result with the interpreter. Compiler build and executable run
resources SHALL have identities and baselines distinct from interpreter
execution.

### TOPAL-COMPILER-SUBSET-001 — Explicit incremental boundary

Until whole-core closure, the compiler SHALL publish its implemented rule set
and SHALL reject any program requiring another rule with stable
`E-COMPILER-UNSUPPORTED` diagnostics. A representation limit SHALL reject only
when the compiler cannot prove conformance; it SHALL NOT wrap, truncate, use
foreign undefined behavior, or change Topal's unbounded semantic domain.

### TOPAL-COMPILER-TOOL-001 — LLVM facility accounting

Every applicable LLVM facility considered for code generation, optimization,
debugging, qualification, profiling, or binary inspection SHALL be recorded as
used, deferred, or not applicable with its correctness and integration reason.
Changing a disposition SHALL update validation evidence in the same increment.
