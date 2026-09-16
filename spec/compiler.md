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

### TOPAL-COMPILER-DIAGNOSTIC-CONTROL-001 — Static diagnostic controls

The compiler SHALL accept warning-specific and structured diagnostic-control
statements after the shared syntax layer has validated their identity, lexical
stack, and next-statement discipline under `TOPAL-SYN-DIAG-001`. A malformed
control SHALL retain the shared source diagnostic. A valid control SHALL NOT
alter source evaluation, evidence trust, value representation, or generated
control flow, and SHALL NOT suppress a language error.

When the compiler emits a configurable warning or severity-neutral diagnostic,
it SHALL apply the active source identity and lexical extent before publishing
that diagnostic. When no such diagnostic is emitted, the checked model SHALL
erase the control before LLVM lowering. This erasure SHALL require no runtime
diagnostic state, foreign dependency, C/C++ runtime, other-language standard
library, public ABI, or native ABI revision. LLVM SHALL receive no operation for
the control itself.

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

### TOPAL-COMPILER-MODULAR-001 — Nominal modular-number lowering

Each admitted root-scope `ModNat` or `ModInt` declaration SHALL retain its
nominal identity and finite inclusive canonical range. Checked construction
SHALL directly admit an `Int` when compile-time evidence proves it is in range;
other admitted inputs SHALL follow
`TOPAL-COMPILER-MODULAR-CONSTRUCTION-001` rather than being wrapped, truncated,
or trapped.
Explicit `value modulo Type` construction SHALL reduce any admitted Int to the
unique canonical representative.

Addition, subtraction, multiplication, and negation over two values of the
same modular type SHALL perform the corresponding exact unbounded Int
operation and then reduce the result into that type's canonical range.
Equality, ordering, and three-way comparison SHALL compare canonical
representatives and SHALL reject operands of different nominal types. These
semantics are mandatory at `-O0` and SHALL NOT depend on machine-integer
overflow or an LLVM optimization.

The private representation MAY reuse the canonical arbitrary-precision Int
pointer, while checked IR, display, DWARF, and GDB SHALL preserve the modular
type's nominal identity. Private definitions, calls, returns, and joins SHALL
use one exact opaque-pointer signature and leave physical AMD64 lowering to
LLVM. This representation SHALL NOT define a foreign or public numeric ABI,
serialization layout, or compiled-library metadata identity and SHALL add no
foreign runtime, other-language standard library, or native-ABI revision.

### TOPAL-COMPILER-MODULAR-CONSTRUCTION-001 — Dynamic checked modular construction

An admitted root binding initialized by a closed finite inclusive `Range Int`
MAY supply a later root modular declaration's range operand. The compiler SHALL
resolve such bindings in source order, retain the same exact canonical bounds
as direct range syntax, and reject a range that is dynamic, malformed, cyclic,
forward-referenced, or otherwise unavailable at the modular declaration.

Checked `Name value` construction SHALL evaluate `value` exactly once. Static
range evidence wholly inside Name's inclusive bounds SHALL produce the nominal
value directly. A syntactically closed value proved wholly outside the bounds
SHALL receive `E-MODULAR-OUT-OF-RANGE`. Otherwise construction SHALL compare
the exact arbitrary-precision Int against both bounds at run time and produce
`Result (Name, lang arithmetic ArithmeticErrorCode)`: success retains the
original canonical Int pointer, while failure contains `out-of-range`, lexical
domain `root.Name(Int)`, and the operand's source file, line, and column.

The dynamic Result SHALL compose through admitted function parameters,
returns, projections, decisions, display, and Error field observation using
the existing private Result/Error headers. DWARF SHALL retain the Result's
source classifier and connect its success payload to Name's distinct nominal
modular type so GDB can safely render either alternative. This lowering SHALL
remain freestanding and SHALL NOT add a foreign runtime, other-language
standard library, public ABI, serialization contract, or native-ABI revision.

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

### TOPAL-COMPILER-ERROR-OPTIONAL-FIELDS-001 — Optional Error provenance fields

Selecting `detail`, `cause`, or `source` from an admitted Error SHALL return,
respectively, `Optional String`, `Optional Error`, or
`Optional SourceLocation`. A null stored detail or cause SHALL become the
corresponding nominal `None`; generated code SHALL NOT synthesize either
payload. Missing source provenance SHALL likewise become `None`.

Present source provenance SHALL become an immutable SourceLocation containing
one-based `line` and `column` fields represented as canonical arbitrary-
precision Int values. Selection SHALL preserve the original Error unchanged,
and Optional decisions, private function boundaries, canonical display,
DWARF, and the bundled GDB renderer SHALL retain each precise payload type.
These semantics are mandatory at `-O0`; LLVM MAY lower private calls and
branches but SHALL NOT infer Topal field meaning. Runtime storage SHALL use
only the Topal Linux platform allocator and SHALL introduce no foreign runtime,
standard library, public ABI, or native-ABI revision.

### TOPAL-COMPILER-ERROR-CODE-001 — Qualified arithmetic code identity

Each qualified value in the closed `lang arithmetic ArithmeticErrorCode`
vocabulary SHALL use the same nominal identity, private tag, display label, and
DWARF enumerator whether constructed directly or observed from an Error. Direct
values SHALL support same-type equality and admitted scalar function passage.
Generated code SHALL NOT perform a runtime namespace lookup or introduce a
foreign runtime dependency for a statically qualified code.

### TOPAL-COMPILER-GENERATOR-ERROR-CODE-001 — Qualified generator code value

The compiler SHALL resolve exactly `lang generator generator-closed` as the
sole initial alternative of the nominal enum type
`lang generator GeneratorErrorCode`. It SHALL preserve that identity through
immutable bindings, same-type equality, decomposed products, canonical display,
DWARF, and GDB. An unknown or incompletely qualified name SHALL NOT be
reinterpreted as a generator error code or as the distinct arithmetic
`ErrorCode` vocabulary.

On Linux x86-64, the backend MAY lower the closed value to private enum tag
zero. Ordinary construction and observation of this value SHALL NOT allocate,
construct, suspend, resume, close, or otherwise control a generator; supply an
intrinsic close result; choose an `Error.domain`; or fabricate generator or
yield provenance. The private tag SHALL NOT become a public, foreign,
persistent, serialized, or compiled-library identity. Future library metadata
SHALL identify the vocabulary and alternative canonically and independently of
the target tag. Generator classifiers and function boundaries, generator
execution and storage, close delivery and handling, and Error integration
remain unsupported. This increment SHALL require no generator runtime,
allocator, foreign dependency, C/C++ runtime, other-language standard library,
needed library, dynamic relocation, or native ABI revision.

### TOPAL-COMPILER-GENERATOR-ITERATE-CONSTRUCT-001 — Lazy iterate construction

The compiler SHALL admit construction of an exact `Generator Int Unit Unit`
from an `Int` initial value and a unary anonymous `Int -> Int` next operation.
It SHALL also admit a directly chained `take-while` with a unary anonymous
`Int -> Boolean` predicate. Construction SHALL evaluate the initial expression
exactly once, retain checked initial, parameter, next-operation, and predicate
structure for later generator lowering, and invoke neither anonymous body.
Final observation SHALL use the canonical `<Generator Int Unit Unit>` display.

This construction-only increment SHALL preserve Generator linearity by
allowing an immutable local Generator binding to be consumed at most once.
Until close delivery and cleanup exist, the compiler SHALL reject a second
read, an unconsumed or explicitly discarded binding, product containment,
qualified member access, equality, decision joins, and function or library
boundaries before LLVM lowering.

On Linux x86-64, the backend MAY represent the admitted live binding as a
private `i32` observation token and SHALL describe that value to DWARF and GDB
as `Generator Int Unit Unit`. The token has no source-level identity and SHALL
NOT encode the initial value, captures, operation identities, continuation
state, ownership, or a stable ABI. Future compiled-library metadata SHALL
instead identify the Generator classifier and captured operations and evidence
canonically, with target-specific representation adapters. This increment
SHALL NOT traverse, yield, resume, suspend, close, allocate generator storage,
emit an indirect call or generator runtime, introduce a foreign dependency or
other-language standard library, or revise `topal-native/6`.

### TOPAL-COMPILER-GENERATOR-UNFOLD-CONSTRUCT-001 — Lazy unfold construction

The compiler SHALL admit construction of an exact `Generator Int Unit Unit`
from a `List Int` seed and a unary anonymous
`List Int -> Optional (Int, List Int)` step operation. Seed and yielded
classifiers SHALL remain distinct. Construction SHALL evaluate the seed
expression exactly once, retain the checked seed, parameter, and step structure
for later generator lowering, and SHALL NOT invoke the step. Final observation
SHALL use the canonical `<Generator Int Unit Unit>` display.

The construction SHALL obey the same one-consumption local linearity boundary
as admitted `iterate` values. In particular, a second read, abandonment,
explicit discard, product containment, qualified access, equality, decision
join, and function or library boundary SHALL be rejected before LLVM lowering.
An anonymous step that captures a Generator SHALL also be rejected.

On Linux x86-64, the backend MAY use the construction-only private `i32`
observation token and semantic `Generator Int Unit Unit` DWARF already defined
for `iterate`. The token SHALL NOT encode the seed, the step, captures,
continuation state, ownership, or a stable ABI. Future compiled-library
metadata SHALL identify the Generator classifier, distinct seed and yield
classifiers, step signature, captured operations, and evidence canonically,
with a target-specific representation adapter. This increment SHALL NOT
traverse, collect, yield, resume, suspend, close, allocate Generator storage,
emit an indirect call or Generator runtime, introduce a foreign dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library Generator ABI, or `topal-native/6` revision.

### TOPAL-COMPILER-GENERATOR-UNFOLD-COLLECT-001 — Finite unfold collection

Unary `collect` SHALL consume an exact `Generator Int Unit Unit` whose retained
construction has an immutable `List Int` seed binding and whose unary step is
exactly `uncons` of its `List Int` parameter. The Generator MAY be consumed
directly or through a chain of linear local bindings. The seed expression SHALL
have been evaluated exactly once at construction. Collection SHALL test the
current seed for `Empty`, append every nonempty seed head to a fresh `List Int`
in order, continue with that seed tail, and terminate without an entry at
`Empty`. It SHALL leave the immutable source List unchanged.

The checked boundary SHALL reject an arbitrary unfold step, step statements or
captures, a seed expression whose already-evaluated identity cannot be retained
by this specialization, and a Generator whose construction provenance is not
available locally. Retained provenance SHALL remain compiler-private and SHALL
NOT be emitted as a persistent, serialized, public, foreign, or library
identity.

On Linux x86-64, the backend SHALL lower the accepted composition as explicit
LLVM control flow over current-seed and result-head/tail SSA values. It MAY
allocate fresh immutable List nodes through the Topal-owned Linux platform
allocator. This lowering is required source-semantic lowering at `-O0`; it
SHALL NOT depend on an LLVM optimization pass. It SHALL NOT materialize an
Optional result, call a List `uncons` helper, use the construction token as
state, allocate a Generator object, emit a callback, indirect call, or generic
Generator runtime, or introduce a foreign dependency, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library Generator ABI, or `topal-native/6` revision. Future
compiled-library metadata SHALL identify the Generator classifier, distinct
seed/yield classifiers, exact step operation and evidence, linear ownership,
and target adapter canonically rather than publish this executable-local
specialization.

### TOPAL-COMPILER-GENERATOR-ITERATE-COLLECT-001 — Finite iterate collection

Unary `collect` SHALL consume a syntactically direct
`Generator Int Unit Unit` produced by `iterate` and bounded by `take-while`.
The compiler SHALL evaluate the initial expression once, test the predicate
once for each candidate, append each accepted Int to a fresh `List Int` in
yield order, and invoke the next operation once after each accepted candidate.
The first rejected candidate SHALL NOT be appended and its next value SHALL
NOT be computed. The resulting List SHALL retain ordinary List equality,
display, private passage, DWARF, and GDB behavior.

The checked boundary SHALL reject collection of an unbounded iterate, an
indirect or previously stored Generator value, and an anonymous next or
predicate body that captures an outer value. The backend SHALL lower the
accepted direct composition as explicit LLVM control flow whose SSA loop state
contains the current arbitrary-precision Int and the private List head/tail.
It MAY allocate the immutable List nodes through the Topal-owned Linux platform
allocator, but SHALL NOT materialize the construction-only Generator token as
continuation state, allocate a Generator object, use host recursion, emit an
indirect call or generic Generator runtime, or introduce a foreign dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library Generator ABI, or `topal-native/6` revision. Future
compiled-library metadata SHALL identify the generator operations and evidence
canonically rather than describe this executable-local specialization.

### TOPAL-COMPILER-GENERATOR-ITERATE-FOREACH-001 — Bounded iterate traversal

A root `foreach` statement SHALL consume an exact `Generator Int Unit Unit`
whose retained construction is an `Int` literal followed by a capture-free
unary `Int -> Int` iterate operation and capture-free unary
`Int -> Boolean` `take-while` predicate. The statement body SHALL bind the
accepted Int to its iteration name, SHALL be capture-free, and SHALL produce
Unit. The statement MAY bind its Unit result, with an optional exact Unit
classifier, or discard that result.

The compiler SHALL test each candidate exactly once before visiting it, execute
the body exactly once for every accepted value in order, and invoke the next
operation exactly once after the body produces Unit. The first rejected
candidate SHALL neither execute the body nor invoke the next operation, and
the traversal SHALL return Unit. The Generator SHALL remain subject to the
one-consumption local linearity boundary.

On Linux x86-64, the backend SHALL lower this behavior as direct LLVM control
flow with the current arbitrary-precision Int as SSA loop state and direct
predicate, body, and next blocks. This is required source-semantic lowering at
`-O0` and SHALL NOT depend on an optimization pass. Generated DWARF SHALL make
the current body binding and Unit result inspectable by GDB. The lowering SHALL
allocate no traversal collection or Generator object, use no construction
token as state, host recursion, callback, indirect call, or generic Generator
runtime, and introduce no foreign dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library Generator
ABI, or `topal-native/6` revision. Future compiled-library metadata SHALL
identify the Generator classifier, operation/predicate/body identities and
evidence, capture layout, linear ownership, and target adapter canonically
rather than publish this executable-local specialization.

### TOPAL-COMPILER-GENERATOR-SINGLE-YIELD-001 — Closed custom suspension

The compiler SHALL admit a root custom generator declaration with exactly one
ordinary Character input, Character yield, Unit resume, and Unit final result
when its body consists of one discarded yield of that input followed by Unit.
Application SHALL evaluate and classify the initial operand once, create a
fresh linear Generator, start execution through the first yield, and retain the
declaration identity and exact yielded Character as compile-session provenance.

Root `foreach` SHALL consume one locally bound admitted instance, invoke its
capture-free Character-to-Unit action exactly once with the yielded value,
resume the generator with Unit, and produce its final Unit. Repeated use and
abandonment other than the exact function-scope close admitted by
`TOPAL-COMPILER-GENERATOR-CLOSE-001` SHALL be rejected under the existing local
linearity boundary.
Declaration shapes not covered by a later compiler rule, overloads, dynamic
Character provenance, direct unbound traversal, captures, and function transfer
except for the exact result admitted by
`TOPAL-COMPILER-CUSTOM-GENERATOR-RESULT-001` or library transfer boundaries
SHALL remain unsupported.

On Linux x86-64, the backend SHALL lower the proven yield, action, Unit resume,
and final Unit as ordered inline code at `-O0`. It MAY use the compiler-private
Generator observation token for ownership and debugging, but SHALL NOT use the
token as continuation state. Generated DWARF SHALL expose the Generator and
yielded Character bindings to GDB. This specialization SHALL allocate no
Generator object or state, call no dispatcher, callback, indirect function, or
foreign runtime, and introduce no C/C++ runtime, other-language standard
library, needed library, dynamic relocation, public/library Generator ABI, or
native-ABI revision. Future compiled-library metadata SHALL identify the
declaration, Generator directions, suspension points, capture/effect evidence,
linear ownership, close behavior, and target adapter canonically.

### TOPAL-COMPILER-GENERATOR-MULTIPLE-YIELD-001 — Repeated custom suspensions

The compiler SHALL generalize the admitted root Character generator to one or
more consecutive discarded yields of its sole initial Character followed by
final Unit. It SHALL retain every yield in source order as compile-session
provenance. Application SHALL evaluate the initial operand exactly once and
stop at the first suspension. Each successful Unit resumption SHALL advance to
the next retained yield or, after the final yield, to the final Unit.

Root `foreach` SHALL consume one locally bound admitted instance and invoke its
capture-free Character-to-Unit action exactly once for every retained yield in
source order. It SHALL resume with Unit after every action and return Unit only
after the final resumption. Exact pre-yield Unit completion SHALL instead obey
`TOPAL-COMPILER-GENERATOR-EARLY-RETURN-001`. Ordinary statements between
yields other than the exact local activation admitted by
`TOPAL-COMPILER-GENERATOR-SUSPENSION-001`, different yield expressions,
overloads, dynamic Character provenance, captures, close handling, and
function or library boundaries SHALL remain unsupported.

On Linux x86-64, the backend SHALL expand the finite proven sequence into
ordered inline action blocks with erased Unit resumptions at `-O0`; this
ordering SHALL NOT depend on LLVM loop unrolling or another optimization pass.
The compiler-private Generator observation token and debug-only Character
shadow MAY be reused, but the token SHALL NOT become continuation state. The
lowering SHALL allocate no Generator object or state, call no dispatcher,
callback, indirect function, or foreign runtime, and introduce no C/C++
runtime, other-language standard library, needed library, dynamic relocation,
public/library Generator ABI, or native-ABI revision. Future compiled-library
metadata SHALL identify the ordered suspension graph and its canonical
declaration/direction, capture/effect, ownership/close, and target-adapter
evidence.

### TOPAL-COMPILER-GENERATOR-LOCAL-BINDING-001 — Exact local state retention

The compiler SHALL admit the existing custom Character-generator subset when
the body begins with exactly one explicitly classified immutable Character
binding initialized from the sole initial Character parameter, followed by one
or more consecutive discarded yields of that local and final Unit. Application
SHALL evaluate the initial operand once, evaluate the alias in generator scope
before the first suspension, and retain its identity and exact Character value
as compile-session provenance. Every retained yield SHALL observe that local,
and the binding SHALL NOT become visible in the caller's checked environment.

Root `foreach` SHALL preserve linear consumption and the existing ordered
action, Unit-resumption, and final-Unit behavior. On Linux x86-64, lowering
SHALL materialize the proven immutable Character value and create a debug-only
pointer shadow for the local in a generator lexical DWARF scope before the
first action. The shadow MAY remain live across the admitted yields but SHALL
be out of scope after traversal. This O0 semantic lowering SHALL NOT depend on
an optimization pass or introduce a Generator object, semantic state
allocation, dispatcher, callback, indirect call, foreign runtime,
other-language standard library, needed library, dynamic relocation,
public/library Generator ABI, or native-ABI revision.

Other local classifiers or initializers, additional statements not admitted by
`TOPAL-COMPILER-GENERATOR-SUSPENSION-001`, dynamic Character provenance,
captures, resume bindings other than the exact Unit success binding admitted by
`TOPAL-COMPILER-GENERATOR-RESUME-BINDING-001`, close handling, and function or
library boundaries SHALL remain unsupported. Future
compiled-library metadata SHALL identify local state canonically with the
declaration, directions, ordered suspension graph, capture/effect evidence,
ownership/close behavior, and target adapters rather than expose the
executable-local debug shadow.

### TOPAL-COMPILER-GENERATOR-EARLY-RETURN-001 — Completion before suspension

The compiler SHALL admit the existing root custom Character-generator
directions when the entire body is final Unit and therefore reaches its
declared Unit result before any yield. Application SHALL evaluate and classify
the initial Character exactly once, create a fresh linear Generator, and retain
an empty suspension sequence plus completed Unit as compile-session provenance.
The absence of a yield SHALL NOT skip static checking of the
Character-to-Unit `foreach` action.

Root `foreach` SHALL consume one locally bound admitted instance, invoke the
action zero times, and produce the generator's final Unit directly. Repeated
use and abandonment SHALL remain rejected under the existing local linearity
boundary. Additional body statements, generator locals, explicit return, other
input/yield/resume/result classifiers, non-Unit final values, overloads,
captures, close handling, and function or library boundaries SHALL remain
unsupported unless another compiler rule admits them.

On Linux x86-64, the backend SHALL evaluate the initial Character once and
lower the empty suspension sequence and final Unit without an action block or
action-parameter debug storage. The compiler-private Generator observation
token MAY remain for ownership and GDB identity, but SHALL NOT represent
continuation state. This O0 semantic lowering SHALL NOT depend on dead-code
elimination or introduce a Generator object, state allocation, dispatcher,
callback, indirect call, foreign runtime, other-language standard library,
needed library, dynamic relocation, public/library Generator ABI, or native-ABI
revision. Future compiled-library metadata SHALL encode the
terminal-before-suspension graph, final value, directions, ownership/close
behavior, and target adapters canonically.

### TOPAL-COMPILER-GENERATOR-FINAL-CHARACTER-001 — Distinct final Character

The compiler SHALL admit one root custom generator with Character input,
Character yield, Unit resume, and Character final result when its body is
exactly one discarded yield of the initial parameter followed by an exact
closed Character literal. Application SHALL evaluate the initial Character
once, retain the yielded value and separate final value as compile-session
provenance, and stop at the yield without evaluating the final expression.

Direct root `foreach` over one locally bound admitted instance SHALL consume
the Generator, invoke its checked Character-to-Unit action exactly once with
the yielded Character, resume with Unit, and only then evaluate and produce the
distinct final Character. Existing repeated-use and abandonment rejection SHALL
continue to apply. Binding the non-Unit `foreach` result, zero or multiple
yields, a local alias, dynamic or non-Character final expressions, other
directions, overloads, captures, close handling, and function or library
boundaries SHALL remain unsupported.

On Linux x86-64, the backend SHALL emit the yield/action/resume sequence before
the final Character materialization at `-O0`. It MAY retain the
compiler-private Generator observation token and existing debug-only yielded
Character shadow, but neither SHALL carry the final value or act as
continuation state. Generated DWARF SHALL expose the full
`Generator Character Unit Character` direction identity and yielded Character
to GDB. The lowering SHALL introduce no Generator object, state allocation,
dispatcher, callback, indirect call, foreign runtime, other-language standard
library, needed library, dynamic relocation, public/library Generator ABI, or
native-ABI revision. Future compiled-library metadata SHALL encode the result
classifier and final-value node alongside canonical declaration, direction,
suspension, effect, ownership/close, and target-adapter evidence.

### TOPAL-COMPILER-GENERATOR-SUSPENSION-001 — Post-resume local activation

The compiler SHALL admit the Unit-final custom Character-generator subset when
its body contains one or more discarded yields of the sole initial Character,
then exactly one explicitly classified immutable Character binding initialized
from that initial parameter, one or more discarded yields of the local, and
final Unit. The checked plan SHALL retain the local identity, classifier,
source span, exact Character value, and the number of successful resumptions
that precede its activation. Starting the generator SHALL stop at the first
yield and SHALL NOT evaluate the post-yield binding.

Direct root `foreach` over one locally bound admitted instance SHALL consume
the Generator, invoke its Character-to-Unit action and resume with Unit for
each retained prefix yield, activate the generator local only after the last
prefix resumption, and then observe the remaining local yields in source order
before final Unit. The local SHALL remain absent from the caller environment.
A local without a later yield, a second local, a non-identity initializer,
yields of the wrong active value, other ordinary statements, dynamic Character
provenance, captures, resume bindings other than the exact Unit success binding
admitted by `TOPAL-COMPILER-GENERATOR-RESUME-BINDING-001`, close handling, and
function or library boundaries SHALL remain unsupported.

On Linux x86-64, O0 lowering SHALL emit the completed prefix action and erased
Unit resumption before the local materialization and its lexical DWARF
declaration, and SHALL emit that declaration before the next yield action. The
checked resumption count SHALL guide source-order expansion but SHALL NOT
become a runtime program counter or public layout. This lowering SHALL NOT
depend on optimization or introduce a Generator object, semantic continuation
state, dispatcher, callback, indirect call, foreign runtime, other-language
standard library, needed library, dynamic relocation, public/library Generator
ABI, or native-ABI revision. Future compiled-library metadata SHALL identify
each local activation transition canonically with the declaration, directions,
ordered suspension graph, capture/effect evidence, ownership/close behavior,
and target adapters.

### TOPAL-COMPILER-GENERATOR-RESUME-BINDING-001 — Exact Unit resumption

The compiler SHALL admit the root custom `Generator Character Unit Unit`
subset when its body is exactly one named, optionally Unit-classified binding
of `yield` applied to the sole initial Character, followed by that binding as
the final Unit expression. The checked plan SHALL retain the yielded Character
and a distinct Unit local whose activation follows one successful resumption.
Starting the generator SHALL stop at the yield without introducing or
evaluating the resume binding.

Direct root `foreach` over one locally bound admitted instance SHALL consume
the Generator, invoke its Character-to-Unit action once, resume with Unit, make
that Unit available under the generator-local name, and only then use it as the
final result. The name SHALL remain absent from the caller environment. A wrong
or discarded binding name, non-Unit classifier, different yielded value,
additional yield or body statement, different final expression, dynamic
Character provenance, captures, close handling, and function or library
boundaries SHALL remain unsupported.

On Linux x86-64, O0 lowering SHALL emit the action before a debug-only `i8`
shadow for the successfully resumed Unit and SHALL expose that binding as Unit
to GDB at its source line. The erased success value SHALL NOT create semantic
continuation state or conflate ordinary Unit resumption with the distinct
`generator-closed` close edge. The lowering SHALL NOT depend on optimization
or introduce a Generator object, state machine, dispatcher, callback, indirect
call, foreign runtime, other-language standard library, needed library,
dynamic relocation, public/library Generator ABI, or native-ABI revision.
Future compiled-library metadata SHALL encode success and close edges,
resume-local activation, the declaration and directions, suspension identity,
capture/effect evidence, ownership/close behavior, and target adapters
canonically.

### TOPAL-COMPILER-GENERATOR-CLOSE-001 — Exact function-scope abandonment

The compiler SHALL admit one call-specialized ordinary function whose body
binds one fresh instance of the exact single-yield custom
`Generator Character Unit Unit` and then reaches final Unit without consuming
it. The initial Character SHALL retain its exact caller provenance. Function
scope exit SHALL consume the suspended Generator, deliver
`generator-closed` with lexical domain `root`, retain the root generator
declaration as separate provenance, and complete the handler-free generator
boundary before the function returns Unit.

The checked function body SHALL contain an explicit custom close after the
Generator binding and before its final expression. Root abandonment, multiple
owned generators, multiple yields, generator locals, and close handlers or
other post-yield work except for the exact handler admitted by
`TOPAL-COMPILER-GENERATOR-CLOSE-HANDLER-001`, dynamic Character provenance
without call specialization, static or anonymous functions, explicit return,
non-Unit final results, Generator parameter/result transfer, and library
boundaries SHALL remain unsupported.

On Linux x86-64, O0 lowering MAY erase the intrinsic close value and final Unit
for this handler-free, effect-free body after preserving their checked order;
the expected close signal has no source observer at the generator boundary.
The compiler-private ownership/debug token SHALL be consumed conceptually and
SHALL remain visible as `Generator Character Unit Unit` in the function's
DWARF scope. The lowering SHALL NOT introduce a Generator object, continuation
state, close dispatcher, callback, indirect call, foreign runtime,
other-language standard library, needed library, dynamic relocation,
public/library Generator ABI, or native-ABI revision. Future compiled-library
metadata SHALL encode the close site and lexical domain, generator declaration
provenance, close/success edges, suspension identity, cleanup/effect evidence,
ownership state, and target adapters canonically.

### TOPAL-COMPILER-GENERATOR-CLOSE-HANDLER-001 — Exact close-result handling

The compiler SHALL generalize the admitted function-local custom close to one
root generator declaration whose body binds its sole Character yield result and
immediately selects a complete `Error`/`Ok` decision with Unit actions. The
checked generator construction SHALL retain the yield-result binding, Error and
Ok binding identities and spans, both branch actions, and the nominal
`lang generator GeneratorErrorCode` set containing `generator-closed`.

When the exact generator is abandoned by the admitted ordinary Unit function,
its checked close SHALL deliver
`Error(domain = root, code = generator-closed)`, select and execute only the
Error action, finish the generator boundary, and then return the function's
final Unit. The successful Unit-resume action SHALL remain explicit metadata
but SHALL NOT execute on this close edge. Root abandonment, successful foreach,
qualified close-code patterns except for the exact matcher admitted by
`TOPAL-COMPILER-GENERATOR-CLOSE-CODE-PATTERN-001`, non-Unit handler actions,
handler work beyond the exact decision, multiple yields or owners, generator
locals, dynamic provenance, transfer boundaries, and library boundaries SHALL
remain unsupported.

On Linux x86-64, O0 lowering SHALL materialize the intrinsic failure through
Topal-owned Result/Error and allocator primitives, statically select its Error
payload, and preserve source order independently of LLVM optimization. DWARF
SHALL expose the yield Result and selected Error with the generator Error-code
vocabulary so GDB renders `generator-closed`, not an unrelated nominal code.
The lowering SHALL NOT introduce a Generator object, continuation state, close
dispatcher, callback, indirect call, unwind dependency, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library Generator ABI, or native-ABI revision. Future compiled-library
metadata SHALL encode handler branches and binding activation together with the
close site/domain, declaration provenance, success/close edges, suspension,
cleanup/effect evidence, ownership state, and target adapters canonically.

### TOPAL-COMPILER-GENERATOR-CLOSE-CODE-PATTERN-001 — Qualified close-code selection

The compiler SHALL generalize the admitted exact close handler to one qualified
`Error ( code is lang generator generator-closed )` Unit rule before its generic
Error fallback, together with the complete Ok Unit rule. The checked handler
SHALL retain the qualified rule's nominal code-set identity, alternative, source
span, and action separately from the generic Error binding/action and Ok
binding/action.

For the statically known close edge, the compiler SHALL select and execute only
the qualified code action. Selection SHALL use the nominal
`lang generator GeneratorErrorCode.generator-closed` identity, not the lexical
Error domain or generator declaration provenance. The generic Error fallback
binding and Ok binding SHALL remain inactive. O0 lowering MAY resolve this known
selection directly without a runtime decision switch, but SHALL still
materialize the Topal-owned Result/Error failure and preserve the nominal code
observation and action source location for GDB.

Other qualified codes or vocabularies, code rules after the generic Error
fallback, missing fallbacks, multiple qualified rules, non-Unit actions,
successful traversal, dynamic close results, multiple yields or owners,
transfer boundaries, and library boundaries SHALL remain unsupported. The
lowering SHALL NOT add a Generator object, continuation state, dispatcher,
callback, indirect call, unwind dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library Generator
ABI, or native-ABI revision. Future compiled-library metadata SHALL preserve
ordered nominal code matchers and actions together with fallback bindings,
success/close edges, domain and declaration provenance, suspension,
cleanup/effect evidence, ownership state, and target adapters canonically.

### TOPAL-COMPILER-CUSTOM-GENERATOR-RESULT-001 — Specialized custom continuation result

The compiler SHALL admit an ordinary nonrecursive called function with exactly
one named Character parameter and result classifier
`Generator Character Unit Unit` when its statement-free body returns a fresh
instance of the exact single-yield custom generator admitted by
`TOPAL-COMPILER-GENERATOR-SINGLE-YIELD-001`, applied directly to that parameter.
The top-level call argument SHALL retain one exact Character. Function exit
SHALL transfer the fresh suspended continuation without close delivery; the
caller SHALL bind and consume it exactly once through the admitted root
Character foreach.

Each call SHALL create a distinct private specialization. The checked program
SHALL retain the generator declaration, exact Character, suspension/final
graph, and ownership transfer as provenance associated with that private
symbol. The caller SHALL evaluate its Character argument once. The callee SHALL
return the compiler-private Generator token, and caller traversal SHALL invoke
the Character-to-Unit action once, resume with Unit, and finish with Unit using
only that call's retained provenance. Distinct calls SHALL NOT share provenance.

On Linux x86-64, LLVM `fastcc` SHALL choose placement for the private Character
descriptor parameter and `i32` Generator result; the compiler SHALL hard-code no
System V register placement. A debug-only Character parameter shadow SHALL keep
the source value inspectable through function return. DWARF and GDB SHALL expose
the Character parameter, Generator result classifier/value, caller traversal
Character, and both call frames.

Static, anonymous, recursive, nested, multiple-parameter, statement-bearing,
non-parameter-derived, multiple-yield, local-state, handled-close, unbound-result,
caller-close, and library paths SHALL remain unsupported. The lowering SHALL
introduce no Generator object or state allocation, dispatcher, callback,
indirect call, unwind dependency, C/C++ runtime, other-language standard
library, needed library, dynamic relocation, public/library calling convention
or Generator ABI, or native-ABI revision. Future compiled-library metadata
SHALL encode the canonical declaration and directions, suspension/final graph,
construction evidence, capture/effect evidence, transfer/ownership/close state,
and target adapters rather than expose the compiler-session side table or
private token.

### TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-001 — Specialized custom continuation parameter

The compiler SHALL admit a top-level call to an ordinary nonrecursive function
with exactly one named `Generator Character Unit Unit` parameter and Unit result
when the argument is a named root binding constructed by the exact single-yield
custom generator admitted by `TOPAL-COMPILER-GENERATOR-SINGLE-YIELD-001`. The
function's executable body SHALL consist only of one Character foreach over that
parameter with a Character-to-Unit action and final Unit. Argument evaluation
SHALL transfer the suspended continuation, consume the caller binding, and make
the parameter its sole owner. Successful traversal SHALL invoke the action once,
resume with Unit, reach final Unit, and return without close delivery.

Each call SHALL create a distinct private specialization. The checked program
SHALL map that call's retained declaration, exact Character, suspension/final
graph, and ownership edge to the parameter only while checking its body, then
restore the surrounding compiler-session provenance. Distinct calls SHALL NOT
share Characters or continuation provenance, and a caller SHALL NOT reuse the
consumed binding.

On Linux x86-64, LLVM `fastcc` SHALL choose placement for the private `i32`
Generator token parameter; the compiler SHALL hard-code no System V register
placement. DWARF and GDB SHALL expose the Generator parameter, yielded Character,
and caller/callee frames. The token SHALL carry no public or semantic
continuation state.

Static, anonymous, recursive, nested, multiple-parameter, additional-body,
direct-expression-argument, multiple-yield, local-state, handled-close,
unconsumed-parameter, returned-parameter, repeated-use, and library paths SHALL
remain unsupported. The lowering SHALL introduce no Generator object or state
allocation, dispatcher, callback, indirect call, unwind dependency, C/C++
runtime, other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or native-ABI revision.
Future compiled-library metadata SHALL encode the canonical declaration and
directions, suspension/final graph, construction evidence, parameter transfer
site, capture/effect evidence, ownership/consumption/close state, action evidence,
and target adapters rather than expose the compiler-session side table or
private token.

### TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-CLOSE-001 — Closing a transferred custom parameter

The compiler SHALL extend the exact custom Generator parameter specialization
of `TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-001` to a statement-free Unit
function body that leaves the transferred parameter unconsumed. Function exit
SHALL consume and close that same suspended continuation. The caller binding
SHALL remain consumed, and neither caller nor callee SHALL subsequently traverse
or resume it.

The checked close node SHALL retain the callee parameter together with the
call-specialized custom construction provenance: declaration identity, exact
Character, directions, suspension/final graph, and ownership edge. Its close
domain SHALL be the lexical root namespace of the admitted function, separate
from the qualified generator declaration provenance. Distinct calls SHALL retain
distinct close provenance.

Because the admitted yield result is discarded and the exact continuation has
no handler, local state, cleanup, effect, or work after suspension, O0 lowering
MAY erase close delivery and final Unit after proving the explicit close node.
This erasure is semantic lowering, not an LLVM optimization. On Linux x86-64,
LLVM `fastcc` SHALL choose placement for the private `i32` Generator token
parameter; the compiler SHALL hard-code no System V register placement. DWARF
and GDB SHALL preserve the Generator parameter and caller/callee frames through
the close edge.

Handled or bound-yield close, multiple yields, local state, cleanup/effects,
non-Unit results, explicit return, additional body statements, traversal before
exit, static/anonymous/recursive/nested/multiple-parameter calls,
direct-expression arguments, repeated caller use, and library paths SHALL remain
unsupported. The lowering SHALL introduce no Generator object or state
allocation, close dispatcher, callback, indirect call, unwind dependency, C/C++
runtime, other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or native-ABI revision.
Future compiled-library metadata SHALL encode the canonical declaration and
directions, suspension/final graph, construction evidence, parameter transfer
and close sites, lexical close domain, capture/effect/cleanup evidence,
ownership/consumption/close state, and target adapters rather than expose the
compiler-session provenance or private token.

### TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-PARAMETER-001 — Transferred final Character

The compiler SHALL extend the exact custom Generator parameter specialization
of `TOPAL-COMPILER-CUSTOM-GENERATOR-PARAMETER-001` to one named
`Generator Character Unit Character` parameter and Character function result.
The argument SHALL be a named root binding of the exact single-yield,
distinct-final-Character generator admitted by
`TOPAL-COMPILER-GENERATOR-FINAL-CHARACTER-001`. The function body SHALL be
exactly a result-valued Character foreach over that parameter with a
Character-to-Unit action. Argument evaluation SHALL consume the caller binding
and transfer sole ownership to the callee.

Successful traversal SHALL invoke the action once with the retained yielded
Character, resume with Unit, evaluate the separately retained final Character,
and return that final Character as the ordinary function result. It SHALL NOT
deliver close. The checked program SHALL preserve the exact Generator
classifier, declaration, yield and final values, suspension/resumption order,
action, and ownership transfer for each private call specialization. Distinct
calls SHALL NOT share yielded or final provenance, and the caller SHALL NOT
reuse the consumed Generator binding.

On Linux x86-64, LLVM `fastcc` SHALL choose placement for the private `i32`
Generator token parameter and ordinary Character descriptor result; the
compiler SHALL hard-code no System V register or return placement. O0 lowering
SHALL expand the action and Unit resumption before materializing and returning
the final Character, independently of LLVM optimization. DWARF and GDB SHALL
expose the complete Generator classifier, callee-owned parameter, yielded
Character, final Character result, and caller/callee frames.

Static, anonymous, recursive, nested, multiple-parameter, additional-body,
direct-expression-argument, multiple-yield, generator-local, handled or
unconsumed, alternate-result, repeated-use, returned-continuation, and library
paths SHALL remain unsupported. The lowering SHALL introduce no Generator
object or state allocation, dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator ABI, or
native-ABI revision. Future compiled-library metadata SHALL encode the
canonical classifier and directions, declaration, distinct yield/final graph,
construction and parameter-transfer sites, action, capture/effect evidence,
ownership/consumption/close state, and target adapters rather than expose the
compiler-session specialization or private token.

### TOPAL-COMPILER-CUSTOM-GENERATOR-CHARACTER-RESULT-001 — Returning a continuation with final Character

The compiler SHALL extend the exact custom Generator result specialization of
`TOPAL-COMPILER-CUSTOM-GENERATOR-RESULT-001` to an ordinary nonrecursive
function whose result classifier is `Generator Character Unit Character`. The
function SHALL have exactly one named Character parameter and a statement-free
body that directly applies the exact single-yield, distinct-final-Character
generator admitted by `TOPAL-COMPILER-GENERATOR-FINAL-CHARACTER-001` to that
parameter. The top-level call SHALL supply one exact Character and bind the
returned continuation before consuming it.

Function exit SHALL transfer the fresh continuation without close delivery.
The checked program SHALL associate the generator declaration, exact yielded
and final Characters, suspension/resumption graph, and ownership transfer with
the private call specialization. Caller traversal SHALL invoke the
Character-to-Unit action once, resume with Unit, then produce the separately
retained final Character. The caller binding SHALL be consumed exactly once,
and no result provenance SHALL be shared with another private specialization.

On Linux x86-64, LLVM `fastcc` SHALL choose placement for the ordinary
Character descriptor parameter and private `i32` Generator result; the compiler
SHALL hard-code no System V register or return placement. The factory SHALL
return only the private ownership token. O0 caller lowering SHALL expand the
retained action and Unit resumption before materializing the final Character,
independently of LLVM optimization. DWARF and GDB SHALL expose the Character
factory parameter, complete Generator return classifier and value, yielded
Character, and caller/factory frames.

Static, anonymous, recursive, nested, multiple-parameter, statement-bearing,
non-parameter-derived, multiple-yield, generator-local, handled-close,
unbound-result, caller-close, parameter-transfer composition, repeated-use, and
library paths SHALL remain unsupported. The lowering SHALL introduce no
Generator object or state allocation, dispatcher, callback, indirect call,
unwind dependency, C/C++ runtime, other-language standard library, needed
library, dynamic relocation, public/library calling convention or Generator
ABI, or native-ABI revision. Future compiled-library metadata SHALL encode the
canonical classifier and directions, declaration, distinct yield/final graph,
construction and function-result transfer sites, action, capture/effect
evidence, ownership/consumption/close state, and target adapters rather than
expose the compiler-session returned-provenance table or private token.

### TOPAL-COMPILER-GENERATOR-STRING-INPUT-001 — Independent String initial input

The compiler SHALL admit a root custom generator whose one named initial
parameter is String while its directions are `Generator Character Unit Unit`.
Its body SHALL bind the result of `empty? initial` to one named Boolean before
one discarded yield of an exact Character literal and a final Unit expression.
One currently admitted String expression SHALL start the generator, and the
fresh result SHALL be bound and consumed exactly once by a Character-to-Unit
foreach action.

The checked program SHALL retain the String parameter independently of the
three Generator directions, the ordered pre-suspension block and Boolean
binding, the exact yielded Character, declaration and suspension spans, final
Unit, and the ownership edge. Generator application SHALL evaluate the String
operand once and execute its emptiness predicate before exposing the suspended
Generator binding. Traversal SHALL then invoke the action once, resume with
Unit, and complete with Unit. These steps SHALL hold with LLVM optimization
disabled and SHALL NOT depend on constant folding or dead-code elimination.

On Linux x86-64, the String descriptor and Boolean prefix value SHALL use the
existing `topal-native/6` representations, while the root-local Generator SHALL
remain a compiler-private `i32` ownership token. LLVM SHALL select target data
layout and instruction placement; the compiler SHALL hard-code no AMD64
register convention. DWARF and GDB SHALL expose the String initial value while
the prefix executes, the Boolean prefix binding metadata, the complete
Generator value, the yielded Character, and the Topal entry frame.

Other initial classifiers, multiple parameters, alternate prefix operations,
unnamed or differently classified prefix bindings, computed or non-Character
yields, multiple yields, non-Unit resume or final directions, local state after
the yield, close handling, nested or ordinary-function construction,
Generator parameter/result transfer, repeated consumption, abandonment,
libraries, and external boundaries SHALL remain unsupported. The lowering
SHALL introduce no Generator object or semantic state allocation, dispatcher,
callback, indirect call, unwind dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library calling
convention or Generator ABI, or native-ABI revision. Future compiled-library
metadata SHALL encode the initial classifier separately from all three
directions, the ordered pre-suspension operation and binding, declaration,
suspension/yield/final graph, construction/action sites, capture/effect
evidence, ownership/consumption/close state, and target adapters rather than
expose the private token or checked-program node layout.

### TOPAL-COMPILER-GENERATOR-STRING-YIELD-001 — Independent String yield direction

The compiler SHALL admit a root custom generator with one named String initial
parameter and directions `Generator String Unit Unit`. Its body SHALL contain
one or more consecutive discarded yields, each yielding either that initial
parameter or an exact String literal, followed by a final Unit expression. One
currently admitted String expression SHALL start the generator, and the fresh
result SHALL be bound and consumed exactly once by a String-to-Unit foreach
action.

The checked program SHALL retain the initial classifier separately from the
three Generator directions and SHALL retain ordered per-yield value provenance,
declaration and suspension spans, final Unit, action, and ownership edge.
Application SHALL evaluate the initial String expression exactly once.
Traversal SHALL deliver that retained descriptor for an initial-parameter yield,
materialize each exact literal at its suspension point, invoke the action and
resume with Unit after every yield, and complete with Unit. This source order
SHALL hold with LLVM optimization disabled and SHALL NOT depend on constant
folding, inlining, or dead-code elimination.

On Linux x86-64, yielded values SHALL use the existing `topal-native/6` String
descriptor while the root-local Generator remains a compiler-private `i32`
ownership token. LLVM SHALL select target data layout and instruction placement;
the compiler SHALL hard-code no AMD64 register convention. DWARF and GDB SHALL
expose the complete Generator and each yielded String while its action executes,
plus the Topal entry frame.

Other input, yield, or resume classifiers; final classifiers beyond the exact
String result admitted by `TOPAL-COMPILER-GENERATOR-FINAL-STRING-001`; multiple
parameters; computed yields; intervening or post-suspension body state beyond
the exact one-discard continuation admitted by
`TOPAL-COMPILER-GENERATOR-RESUME-DISCARD-001`; close handling; nested or
ordinary-function construction; Generator parameter/result transfer; repeated
consumption; abandonment; libraries; and external boundaries SHALL remain
unsupported. The lowering SHALL introduce no Generator object or semantic
state allocation, dispatcher, callback, indirect call, unwind dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library calling convention or Generator ABI, or native-ABI
revision.
Future compiled-library metadata SHALL encode the initial and direction
classifiers independently, ordered per-yield values and provenance, declaration,
suspension/final graph, construction/action sites, capture/effect evidence,
ownership/consumption/close state, and target adapters rather than expose the
private token or checked-program node layout.

### TOPAL-COMPILER-GENERATOR-FINAL-STRING-001 — Distinct final String

The compiler SHALL admit a root custom generator with one named String initial
parameter and directions `Generator String Unit String`. Its body SHALL contain
one or more consecutive discarded String yields in the forms admitted by
`TOPAL-COMPILER-GENERATOR-STRING-YIELD-001`, followed by one exact String literal
as its distinct final expression. One currently admitted String expression SHALL
start the generator, and the fresh result SHALL be bound and consumed exactly
once by a String-to-Unit foreach action whose expression result is the final
String.

The checked program SHALL retain the final String expression and classifier
separately from the initial value, ordered yield provenance, Unit resume
direction, declaration and suspension spans, action, and ownership edge.
Application SHALL evaluate the initial expression exactly once. Traversal SHALL
deliver every yield, invoke its action, resume with Unit, and only then
materialize and return the final String. This order SHALL hold with LLVM
optimization disabled and SHALL NOT depend on folding, inlining, or dead-code
elimination.

On Linux x86-64, the input, yielded, and final values SHALL use the existing
`topal-native/6` String descriptor while the root-local Generator remains a
compiler-private `i32` ownership token. LLVM SHALL select target data layout and
instruction placement; the compiler SHALL hard-code no AMD64 register
convention. DWARF and GDB SHALL expose the complete
`Generator String Unit String` classifier and value, the yielded String during
its action, the final expression source location, and the Topal entry frame.

Nonliteral or initial-derived final values; other input, yield, resume, or final
classifiers; multiple parameters; computed yields; intervening body state;
close handling; nested or ordinary-function construction; Generator
parameter/result transfer; repeated consumption; abandonment; libraries; and
external boundaries SHALL remain unsupported. The lowering SHALL introduce no
Generator object or semantic state allocation, dispatcher, callback, indirect
call, unwind dependency, C/C++ runtime, other-language standard library, needed
library, dynamic relocation, public/library calling convention or Generator
ABI, or native-ABI revision. Future compiled-library metadata SHALL encode the
initial and direction classifiers independently, ordered yield provenance, the
distinct final-value expression and provenance, declaration, suspension/final
graph, construction/action sites, capture/effect evidence,
ownership/consumption/close state, and target adapters rather than expose the
private token or checked-program node layout.

### TOPAL-COMPILER-GENERATOR-RESUME-DISCARD-001 — Post-resume discarded computation

The compiler SHALL admit a root custom generator with one named String initial
parameter and directions `Generator String Unit Unit` whose body extends the
yield forms of `TOPAL-COMPILER-GENERATOR-STRING-YIELD-001` with exactly one
discarded `empty? initial` computation. At least one yield SHALL precede that
computation and at least one yield SHALL follow it. The body SHALL end in Unit.
One currently admitted String expression SHALL start the generator, and the
fresh result SHALL be bound and consumed exactly once by a String-to-Unit
foreach action.

The checked program SHALL retain the computation as a typed continuation block
with its source span and exact successful-resumption ordinal, separately from
the initial value, ordered yield provenance, action, final Unit, and ownership
edge. Application SHALL evaluate the initial expression exactly once. Traversal
SHALL deliver and act on each preceding yield, resume it with Unit, execute the
discarded emptiness computation against the captured initial String, and only
then reach the following suspension. This order SHALL hold with LLVM
optimization disabled and SHALL NOT depend on folding, inlining, or dead-code
elimination.

On Linux x86-64, the initial and yielded values SHALL use the existing
`topal-native/6` String descriptor while the root-local Generator remains a
compiler-private `i32` ownership token. LLVM SHALL select target data layout and
instruction placement; the compiler SHALL hard-code no AMD64 register
convention. DWARF and GDB SHALL expose the complete
`Generator String Unit Unit` classifier and value, each yielded String during
its action, the captured initial String during the post-resume computation, its
source location, and the Topal entry frame.

Bindings, more than one continuation computation, a computation before the
first or after the final yield, operands other than the initial parameter,
computed continuation expressions, non-Unit final values, other input, yield,
or resume classifiers, close handling, nested or ordinary-function
construction, Generator parameter/result transfer, repeated consumption,
abandonment, libraries, and external boundaries SHALL remain unsupported. The
lowering SHALL introduce no Generator object or semantic state allocation,
dispatcher, callback, indirect call, unwind dependency, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or native-ABI revision.
Future compiled-library metadata SHALL encode the initial and direction
classifiers independently, ordered yields, the continuation block and
successful-resumption ordinal, expression provenance and source sites,
declaration/suspension/final graph, action, capture/effect evidence,
ownership/consumption/close state, and target adapters rather than expose the
private token or checked-program node layout.

### TOPAL-COMPILER-GENERATOR-EXPLICIT-RETURN-001 — Explicit return before suspension

The compiler SHALL admit a root custom generator with one named String initial
parameter and directions `Generator String Unit String` whose body consists
only of `return` applied to one exact String literal. One currently admitted
String expression SHALL start the generator, and the fresh result SHALL be
bound and consumed exactly once by a String-to-Unit foreach action whose
expression result is the explicitly returned String.

The checked program SHALL distinguish the explicit-return keyword and source
site from the typed String result expression, implicit final expressions,
empty ordered yield/continuation lists, action, and ownership edge. Application
SHALL evaluate the initial expression exactly once even when the return does
not reference it. Traversal SHALL recognize completion before the first
suspension, invoke the foreach action zero times, and directly materialize and
return the exact String. This order SHALL hold with LLVM optimization disabled
and SHALL NOT depend on folding, inlining, or dead-code elimination.

On Linux x86-64, the initial and returned values SHALL use the existing
`topal-native/6` String descriptor while the root-local Generator remains a
compiler-private `i32` ownership token. A debug-only pointer shadow MAY keep the
in-scope initial parameter inspectable without becoming semantic Generator
state. LLVM SHALL select target data layout and instruction placement; the
compiler SHALL hard-code no AMD64 register convention. DWARF and GDB SHALL
expose the complete `Generator String Unit String` classifier and value, the
initial String at the explicit-return source location, and the Topal entry
frame. They SHALL NOT fabricate a yielded action value when no suspension is
reached.

Implicit zero-yield final expressions, nonliteral or initial-derived returns,
explicit return after a suspension beyond the exact case admitted by
`TOPAL-COMPILER-GENERATOR-RETURN-AFTER-YIELD-001`, Unit or other return
classifiers, additional body statements, multiple parameters, other input,
yield, or resume classifiers, close handling, nested or ordinary-function
construction, Generator parameter/result transfer, repeated consumption,
abandonment, libraries, and external boundaries SHALL remain unsupported. The
lowering SHALL introduce no Generator object or semantic state allocation,
dispatcher, callback, indirect call, unwind dependency, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or native-ABI revision.
Future compiled-library
metadata SHALL encode explicit versus implicit completion, return expression
and keyword provenance, reachability and empty-yield evidence, independent
initial and direction classifiers, declaration/construction/action sites,
captures/effects, ownership/consumption/close state, and target adapters rather
than expose the private token, debug shadow, or checked-program node layout.

### TOPAL-COMPILER-GENERATOR-RETURN-AFTER-YIELD-001 — Explicit return after resumption

The compiler SHALL admit a root custom generator with one named String initial
parameter and directions `Generator String Unit String` whose body consists
only of one discarded `yield initial` followed by `return` applied to one exact
String literal. One currently admitted String expression SHALL start the
generator, and the fresh result SHALL be bound and consumed exactly once by a
String-to-Unit foreach action whose expression result is the explicitly
returned String.

The checked program SHALL retain the initial-parameter yield and suspension,
the explicit-return keyword and source site, and the typed final String as
separate ordered provenance. Application SHALL evaluate the initial expression
exactly once. Traversal SHALL invoke the action exactly once with that retained
String, resume the generator with Unit, and only then materialize and return the
exact String literal. This order SHALL hold with LLVM optimization disabled and
SHALL NOT depend on folding, inlining, or dead-code elimination.

On Linux x86-64, the initial, yielded, and returned values SHALL use the
existing `topal-native/6` String descriptor while the root-local Generator
remains a compiler-private `i32` ownership token. A debug-only pointer shadow
MAY keep the initial parameter inspectable at the return site without becoming
semantic Generator state. LLVM SHALL select target data layout and instruction
placement; the compiler SHALL hard-code no AMD64 register convention. DWARF
and GDB SHALL expose the complete `Generator String Unit String` classifier and
value, the yielded String during its action, the initial String at the explicit
return, and the Topal entry frame.

Literal or computed yields other than the initial parameter, multiple yields,
nonliteral or initial-derived returns, intervening statements, Unit or other
direction classifiers, multiple parameters, close handling, nested or
ordinary-function construction, Generator parameter/result transfer, repeated
consumption, abandonment, libraries, and external boundaries SHALL remain
unsupported. The lowering SHALL introduce no Generator object or semantic
state allocation, dispatcher, callback, indirect call, unwind dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library calling convention or Generator ABI, or native-ABI
revision. Future compiled-library metadata SHALL encode explicit completion,
ordered yield/resume/return provenance and reachability, independent initial
and direction classifiers, declaration/construction/action sites,
captures/effects, ownership/consumption/close state, and target adapters rather
than expose the private token, debug shadow, or checked-program node layout.

### TOPAL-COMPILER-GENERATOR-BOOLEAN-001 — Boolean generator directions

The compiler SHALL admit a root custom generator with one named Boolean initial
parameter and directions `Generator Boolean Unit Boolean` whose body consists
only of one discarded `yield initial` followed by `not initial` as its distinct
final expression. One currently admitted Boolean expression SHALL start the
generator, and the fresh result SHALL be bound and consumed exactly once by a
Boolean-to-Unit foreach action whose expression result is the final Boolean.

The checked program SHALL retain the Boolean initial classifier separately from
all three Generator directions, the initial-parameter yield and suspension, the
typed final negation, action, and ownership edge. Application SHALL evaluate
the initial expression exactly once. Traversal SHALL invoke the action exactly
once with that retained Boolean, resume the generator with Unit, and only then
evaluate the final negation against the captured initial value. This order
SHALL hold with LLVM optimization disabled and SHALL NOT depend on folding,
inlining, or dead-code elimination.

Unoptimized LLVM IR SHALL retain the action negation before the final negation.
Mandatory target instruction selection MAY omit the unused action result only
because Boolean negation is total and effect-free; this SHALL NOT generalize to
an action with an observable effect. The independently emitted debug lifetime
anchor SHALL keep the yielded value inspectable even when no machine
instruction is selected for that pure result.

On Linux x86-64, each Boolean SHALL use LLVM `i1` in the existing private
`topal-native/6` representation while the root-local Generator remains a
compiler-private `i32` ownership token. A debug-only aligned `i1` stack shadow
MAY anchor the yielded value's source lifetime without becoming semantic
Generator state. LLVM SHALL select physical register or stack placement and
instruction selection; the compiler SHALL hard-code no AMD64 register
convention. DWARF and GDB SHALL expose the complete
`Generator Boolean Unit Boolean` classifier and value, the yielded Boolean in
the foreach action scope, the captured initial Boolean at the final expression,
and the Topal entry frame.

Literal, computed, or multiple yields; a final expression other than
`not initial`; additional body statements; multiple parameters; other input,
yield, resume, or final classifiers; close handling; nested or
ordinary-function construction; Generator parameter/result transfer; repeated
consumption; abandonment; libraries; and external boundaries SHALL remain
unsupported. The lowering SHALL introduce no Generator object or semantic
state allocation, Boolean helper, dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator ABI, or
native-ABI revision. Future compiled-library metadata SHALL encode independent
initial and direction classifiers, ordered Boolean expression and
yield/resume/final provenance, declaration/construction/action sites,
captures/effects, ownership/consumption/close state, and target adapters rather
than expose the private token, debug shadow, or checked-program node layout.

### TOPAL-COMPILER-GENERATOR-INT-001 — Arbitrary-precision Int generator directions

The compiler SHALL admit a root custom generator with one named Int initial
parameter and directions `Generator Int Unit Int` whose body consists only of
one discarded `yield initial` followed by `initial + 1` as its distinct final
expression. One currently admitted Int expression SHALL start the generator,
and the fresh result SHALL be bound and consumed exactly once by an Int-to-Unit
foreach action consisting only of a discarded `value + 1`.

The checked program SHALL retain the Int initial classifier separately from
all three Generator directions, the initial-parameter yield and suspension,
both typed additions and their exact-one operands, action, and ownership edge.
Application SHALL evaluate the initial expression exactly once. Traversal SHALL
invoke the action addition exactly once with that retained Int, resume the
generator with Unit, and only then evaluate the final addition against the
captured initial value. Both additions SHALL use the canonical arbitrary-
precision Int runtime and preserve all values exactly. This order and the
explicit allocation-failure behavior SHALL hold with LLVM optimization
disabled and SHALL NOT depend on folding, inlining, or dead-code elimination.

Unoptimized LLVM IR SHALL retain two direct Int-addition calls in source order.
Unlike the total register-only Boolean action in
`TOPAL-COMPILER-GENERATOR-BOOLEAN-001`, LLVM SHALL NOT omit the discarded Int
action: arbitrary-precision addition can allocate and therefore can reach the
required platform-failure path. LLVM MAY optimize inside or around the calls
only when exact values, evaluation order, and allocation failure remain
observationally equivalent.

On Linux x86-64, every Int SHALL retain the existing private immutable
`topal-native/6` canonical sign-and-magnitude pointer representation while the
root-local Generator remains a compiler-private `i32` ownership token. LLVM
SHALL select pointer placement and call lowering from the target triple and
data layout; the compiler SHALL hard-code no AMD64 register convention. Two
debug-only aligned pointer slots MAY anchor the yielded and captured-initial
source lifetimes without becoming semantic Generator state. DWARF, the Topal
GDB printer, and GDB SHALL expose the complete `Generator Int Unit Int`
classifier and value, the exact yielded Int in the foreach action scope, the
exact captured initial Int at the final expression, and the Topal entry frame.

Literal, computed, or multiple yields; a final expression other than
`initial + 1`; additional body statements; multiple parameters; other input,
yield, resume, or final classifiers; close handling; nested or
ordinary-function construction; Generator parameter/result transfer; repeated
consumption; abandonment; libraries; and external boundaries SHALL remain
unsupported. The lowering SHALL introduce no semantic Generator object or
state allocation, dispatcher, callback, indirect call, unwind dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library calling convention or Generator ABI, or native-ABI
revision. Existing Int values and arithmetic allocations SHALL remain wholly
owned by the Topal runtime and Linux syscall layer. Future compiled-library
metadata SHALL encode independent initial and direction classifiers, ordered
Int expression and yield/resume/final provenance, exact numeric requirements,
declaration/construction/action sites, captures/effects including allocation
failure, ownership/consumption/close state, native-representation identity,
and target adapters rather than expose the private token, debug slots, Int
object layout, or checked-program node layout.

### TOPAL-COMPILER-GENERATOR-RATIONAL-001 — Exact Rational generator directions

The compiler SHALL admit a root custom generator with one named Rational
initial parameter and directions `Generator Rational Unit Rational` whose body
consists only of one discarded `yield initial` followed by
`initial + (Rational (1, 3))` as its distinct final expression. One currently
admitted Rational expression SHALL start the generator, and the fresh result
SHALL be bound and consumed exactly once by a Rational-to-Unit foreach action
consisting only of a discarded `value + (Rational (1, 3))`.

The checked program SHALL retain the Rational initial classifier separately
from all three Generator directions, the initial-parameter yield and
suspension, both typed additions, their exact one-third constructor provenance,
action, and ownership edge. Application SHALL evaluate and canonically
construct the initial expression exactly once. Traversal SHALL construct the
action addend and invoke the action addition exactly once with that retained
Rational, resume the generator with Unit, and only then construct the final
addend and evaluate the final addition against the captured initial value. All
constructors and additions SHALL use the canonical exact Rational and arbitrary-
precision Int runtime. This order and explicit allocation-failure behavior
SHALL hold with LLVM optimization disabled and SHALL NOT depend on folding,
inlining, or dead-code elimination.

Unoptimized LLVM IR SHALL retain three direct canonical Rational-construction
calls and two direct Rational-addition calls in source order. LLVM SHALL NOT
omit the discarded action construction or addition: both may allocate and
therefore can reach the required platform-failure path. LLVM MAY optimize
inside or around these calls only when exact canonical values, evaluation
order, and allocation failure remain observationally equivalent.

On Linux x86-64, every Rational SHALL retain the existing private immutable
`topal-native/6` pointer representation containing canonical arbitrary-
precision Int numerator and positive denominator pointers, while the root-local
Generator remains a compiler-private `i32` ownership token. LLVM SHALL select
pointer placement and call lowering from the target triple and data layout; the
compiler SHALL hard-code no AMD64 register convention. Two debug-only aligned
pointer slots MAY anchor the yielded and captured-initial source lifetimes
without becoming semantic Generator state. DWARF, the Topal GDB printer, and
GDB SHALL expose the complete `Generator Rational Unit Rational` classifier and
value, the exact yielded Rational in the foreach action scope, the exact
captured initial Rational at the final expression, and the Topal entry frame.

Literal, converted, computed, or multiple yields; different Rational addends;
a final expression other than the exact declared addition; additional body
statements; multiple parameters; other input, yield, resume, or final
classifiers; close handling; nested or ordinary-function construction;
Generator parameter/result transfer; repeated consumption; abandonment;
libraries; and external boundaries SHALL remain unsupported. The lowering
SHALL introduce no semantic Generator object or state allocation, dispatcher,
callback, indirect call, unwind dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library calling
convention or Generator ABI, or native-ABI revision. Existing Rational/Int
values and arithmetic allocations SHALL remain wholly owned by the Topal
runtime and Linux syscall layer. Future compiled-library metadata SHALL encode
independent initial and direction classifiers, ordered Rational expression and
yield/resume/final provenance, exact canonical numerator/denominator and
constructor requirements, declaration/construction/action sites,
captures/effects including allocation failure, ownership/consumption/close
state, native-representation identity, and target adapters rather than expose
the private token, debug slots, Rational/Int object layouts, or checked-program
node layout.

### TOPAL-COMPILER-GENERATOR-UNIT-001 — Payload-free Unit generator directions

The compiler SHALL admit a root custom generator with one named Unit initial
parameter and directions `Generator Unit Unit Unit` whose body consists only of
one discarded `yield initial` followed by `()` as its final expression. One
currently admitted Unit expression SHALL start the generator, and the fresh
result SHALL be bound and consumed exactly once by a Unit-to-Unit foreach
action consisting only of its named yielded parameter as an identity
expression.

The checked program SHALL retain the Unit initial classifier separately from
all three Generator directions, the initial-parameter yield and suspension, the
named identity action, Unit resumption, distinct final Unit, and ownership edge.
Application SHALL evaluate the initial expression exactly once. Traversal SHALL
evaluate the action exactly once with the yielded Unit, resume the generator
with Unit, and only then evaluate the final Unit. This order SHALL hold with
LLVM optimization disabled and SHALL NOT depend on folding, inlining, or dead-
code elimination.

Because Unit has exactly one value and no runtime payload, unoptimized semantic
LLVM IR SHALL introduce no action call, payload operation, allocation, or
Generator runtime operation. Erasing those payload operations is a compiler
representation decision, not an LLVM optimization, and SHALL preserve the
checked action occurrence and source order. Debug LLVM IR SHALL contain two
aligned private `i8` lifetime slots: one anchors the yield and action sites, and
one initialized slot anchors the captured initial and final-expression site.
These slots and their stores SHALL be debug-only and SHALL NOT become semantic
Generator or Unit state. LLVM MAY eliminate or transform the debug-only
artifacts when permitted by the selected debug/optimization policy.

On Linux x86-64, Unit SHALL have no semantic native payload while the root-
local Generator remains a compiler-private `i32` ownership token. LLVM SHALL
select placement and call lowering from the target triple and data layout; the
compiler SHALL hard-code no AMD64 register convention. DWARF and GDB SHALL
expose the complete `Generator Unit Unit Unit` classifier and value, the Unit
yielded into the foreach action, the captured initial Unit at the final
expression, ordered yield/action/resumption/final source locations, and the
Topal entry frame.

Literal or multiple yields; a final expression other than exact `()`;
additional body statements; an action other than the named identity expression;
multiple parameters; other input, yield, resume, or final classifiers; close
handling; nested or ordinary-function construction; Generator parameter/result
transfer; repeated consumption; abandonment; libraries; and external
boundaries SHALL remain unsupported. The lowering SHALL introduce no semantic
Generator object or state allocation, dispatcher, callback, indirect call,
unwind dependency, C/C++ runtime, other-language standard library, needed
library, dynamic relocation, public/library calling convention or Generator
ABI, or native-ABI revision. Output SHALL remain wholly owned by the Topal
runtime and Linux syscall layer. Future compiled-library metadata SHALL encode
independent initial and direction classifiers, the zero-payload Unit identity,
ordered yield/action/resume/final provenance, representation erasure guarantees,
declaration/construction/action sites, captures/effects, ownership/consumption/
close state, native-representation identity, and target adapters rather than
expose the private token, debug slots, or checked-program node layout.

### TOPAL-COMPILER-GENERATOR-OPTIONAL-001 — Nominal Optional generator directions

The compiler SHALL admit a root custom generator with one named `Optional Int`
initial parameter and directions `Generator Optional Int Unit Optional Int`
whose body consists only of one discarded `yield initial` followed by
`None Int` as its final expression. One currently admitted `Optional Int`
expression SHALL start the generator; the shared regression uses `(Some 7)`.
The fresh result SHALL be bound and consumed exactly once by a foreach action
consisting only of the discarded equality `candidate = (Some 7)` for its named
yielded parameter.

The checked program SHALL retain the nominal Optional classifier and its Int
payload classifier separately from all three Generator directions, the
initial-parameter yield and suspension, the exact Some construction and
equality action, Unit resumption, distinct final None construction, and
ownership edge. Application SHALL construct the initial Optional exactly once.
Traversal SHALL pass that same immutable Optional value to the action, evaluate
the exact right operand and equality once, resume the generator with Unit, and
only then construct the final None value. This order SHALL hold with LLVM
optimization disabled and SHALL NOT depend on folding, inlining, or dead-code
elimination.

On Linux x86-64, the existing compiler-private Optional representation SHALL
remain an aligned pointer to Topal-owned tagged storage with an Int payload
pointer for Some and a null payload for None. The root-local Generator SHALL
remain a compiler-private `i32` ownership token. LLVM SHALL select placement
and call lowering from the target triple and data layout; the compiler SHALL
hard-code no AMD64 register convention. Two aligned debug-only pointer shadows
SHALL preserve the yielded action value and captured initial lifetime. DWARF
and GDB SHALL expose the complete `Generator Optional Int Unit Optional Int`
classifier and value, the yielded `Some 7`, the captured initial `Some 7`,
ordered yield/action/resumption/final source locations, and the Topal entry
frame.

Literal or multiple yields; a final expression other than exact `None Int`;
additional body statements; another Optional payload, action, input, yield,
resume, or final classifier; close handling; nested or ordinary-function
construction; Generator parameter/result transfer; repeated consumption;
abandonment; libraries; and external boundaries SHALL remain
unsupported. The lowering SHALL introduce no semantic Generator object or
state allocation, dispatcher, callback, indirect call, unwind dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library calling convention or Generator ABI, or native-ABI
revision. Optional storage, allocation, equality, display, and Linux syscalls
SHALL remain wholly owned by Topal. Future compiled-library metadata SHALL
encode independent initial and direction classifiers, the nominal Optional and
payload identities, exact alternatives and action operands, ordered yield/
action/resume/final provenance, allocation-failure effects, declaration and
construction sites, captures/effects, ownership/consumption/close state,
native-representation identity, and target adapters rather than expose the
private token, debug slots, Optional/Int object layouts, or checked-program
node layout.

### TOPAL-COMPILER-GENERATOR-RESULT-001 — Exact Result generator directions

The compiler SHALL admit a root custom generator with one named
`Result (Rational, lang arithmetic ArithmeticErrorCode)` initial parameter and
matching yield and final directions, with Unit resumption. Its body SHALL
consist only of one discarded `yield initial` followed by
`initial / (Rational 0)`. The shared regression SHALL start the generator with
the exact proven-success expression `Rational 1`; the fresh result SHALL be
bound and consumed exactly once by a foreach action consisting only of the
discarded self-equality `candidate = candidate` for its named yielded
parameter.

The checked program SHALL retain the nominal Result and Rational success
identities separately in all three Generator directions, the proven-success
promotion, initial-parameter yield and suspension, reflexive action, Unit
resumption, structured fallible final division, source-located error
provenance, declaration provenance, and ownership edge. Application SHALL
evaluate and promote Rational 1 exactly once. Traversal SHALL pass that same
immutable successful Result to the action, prove its self-equality without
inspecting or duplicating the payload, resume with Unit, and only then perform
the exact division by zero and produce the structured failure Result. This
order SHALL hold with LLVM optimization disabled and SHALL NOT depend on
folding, inlining, or dead-code elimination.

On Linux x86-64, Result and Rational SHALL retain their existing Topal-owned
compiler-private pointer representations, and the root-local Generator SHALL
remain a compiler-private `i32` ownership token. LLVM SHALL derive placement,
alignment, and call lowering from the target triple and data layout; the
compiler SHALL hard-code no AMD64 register convention. Two aligned debug-only
pointer shadows and four lifetime/source anchor stores SHALL keep the yielded
action value and captured initial inspectable. DWARF and GDB SHALL expose the
complete Result-bearing Generator classifier and value, the yielded successful
Rational 1 Result, captured initial Result, ordered yield/action/resumption/
final source locations, and the Topal entry frame.

Another Result success/error classifier, input expression or value, direction,
yield, final operation, divisor, or action; multiple yields; additional body
statements; general inlined Result projection/propagation; close handling;
nested or ordinary-function construction; Generator parameter/result transfer;
repeated consumption; abandonment; libraries; and external boundaries SHALL
remain unsupported. The lowering SHALL introduce no semantic Generator object
or state allocation, dispatcher, callback, indirect call, unwind dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library calling convention or Generator/Result ABI, or
native-ABI revision. Result/Rational storage, allocation, failure construction,
display, and Linux syscalls SHALL remain wholly owned by Topal. Future compiled-
library metadata SHALL encode independent initial and direction classifiers,
nominal Result/success/error identities, success evidence, fallible-operation
and source-error provenance, ordered yield/action/resume/final provenance,
allocation-failure effects, declaration/construction sites, captures/effects,
ownership/consumption/close state, native-representation identity, and target
adapters rather than expose the private token, debug slots, pointer/object
layouts, or checked-program node layout.

### TOPAL-COMPILER-GENERATOR-COMPARISON-001 — Exact Comparison generator directions

The compiler SHALL admit a root custom generator with one named `Comparison`
initial parameter and directions `Generator Comparison Unit Comparison`. Its
body SHALL consist only of one discarded `yield initial` followed by `3 <=> 2`
as its final expression. The shared regression SHALL start the generator with
the exact expression `1 <=> 2`; the fresh result SHALL be bound and consumed
exactly once by a foreach action consisting only of the discarded
`comparison = (1 <=> 2)` for its named yielded parameter.

The checked program SHALL retain the language-defined nominal `Comparison`
identity separately in all three Generator directions, both ordered Int
operands and three-way operations, the initial-parameter yield and suspension,
the exact equality action, Unit resumption, distinct final comparison,
declaration provenance, and ownership edge. Application SHALL evaluate
`1 <=> 2` exactly once. Traversal SHALL pass that same immutable Less value to
the action, independently evaluate the action's `1 <=> 2` once and compare the
two nominal values, resume the generator with Unit, and only then evaluate
`3 <=> 2` to produce Greater. This order SHALL hold with LLVM optimization
disabled and SHALL NOT depend on folding, inlining, or dead-code elimination.

On Linux x86-64, `Comparison` SHALL retain its existing compiler-private signed
`i32` values for Less, Equal, and Greater, and the root-local Generator SHALL
remain a compiler-private `i32` ownership token. LLVM SHALL derive placement,
alignment, and call lowering from the target triple and data layout; the
compiler SHALL hard-code no AMD64 register convention. Two aligned debug-only
`i32` shadows and four lifetime/source anchor stores SHALL keep the yielded
action value and captured initial inspectable. DWARF and GDB SHALL expose the
complete `Generator Comparison Unit Comparison` classifier and value, the
yielded Less value, captured initial Less value, ordered yield/action/
resumption/final source locations, and the Topal entry frame.

Another input, action, final comparison, operand, direction, yield, or value;
multiple yields; additional body statements; close handling; nested or
ordinary-function construction; Generator parameter/result transfer; repeated
consumption; abandonment; libraries; and external boundaries SHALL remain
unsupported. The lowering SHALL introduce no semantic Generator object or
state allocation, dispatcher, callback, indirect call, unwind dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library calling convention or Generator/Comparison ABI, or
native-ABI revision. Int/Comparison allocation, comparison, equality, display,
and Linux syscalls SHALL remain wholly owned by Topal. Future compiled-library
metadata SHALL encode independent initial and direction classifiers, nominal
Comparison identity and ordered alternatives, operand and operation
provenance, ordered yield/action/resume/final provenance, allocation-failure
effects, declaration/construction sites, captures/effects, ownership/
consumption/close state, native-representation identity, and target adapters
rather than expose the private token, debug slots, scalar tags, or checked-
program node layout.

### TOPAL-COMPILER-GENERATOR-NESTED-OPTIONAL-001 — Recursive Optional product directions

The compiler SHALL admit a root custom generator with one named
`Optional (Int, String)` initial parameter and directions
`Generator Optional (Int, String) Unit Optional (Int, String)`. Its body SHALL
consist only of one discarded `yield initial` followed by
`Some (8, "done")`. The shared regression SHALL start the generator with exact
`Some (7, "item")`; the fresh result SHALL be bound and consumed exactly once
by a foreach action consisting only of the discarded
`candidate = (Some (7, "item"))` for its named yielded parameter.

The checked program SHALL retain the nominal Optional identity, Some
alternative, positional-product arity and order, and Int and String field
identities separately in all three Generator directions. It SHALL also retain
the initial-parameter yield and suspension, field-wise Optional equality
action, Unit resumption, distinct final product, declaration provenance, and
ownership edge. Application SHALL evaluate both initial payload fields once in
source order and construct one Optional. Traversal SHALL pass that same
immutable value to the action, construct the action operand once, compare tags
before observing Some payloads, compare payload fields in source order, resume
with Unit, and only then construct `Some (8, "done")`. This order SHALL hold at
LLVM O0 without relying on folding, inlining, or dead-code elimination.

On Linux x86-64, Optional SHALL retain its existing Topal-owned tagged pointer
header. Each admitted Some product payload SHALL use a Topal-owned aligned
16-byte allocation containing the existing Int and String pointers in source
order. The root-local Generator SHALL remain a compiler-private `i32` ownership
token. LLVM SHALL derive placement, alignment, and call lowering from the
target triple and data layout; the compiler SHALL hard-code no AMD64 register
convention. Two aligned debug-only Optional pointer shadows SHALL preserve the
yielded action value and captured initial lifetime. DWARF and GDB SHALL expose
the complete recursive Generator classifier and value, the Optional product
classifier, ordered payload fields, both yielded/captured `Some (7, "item")`
values, ordered yield/action/resumption/final source locations, and the Topal
entry frame.

Another Optional payload, product arity, field order, input, action, final,
direction, yield, or value; None; multiple yields; additional body statements;
close handling; nested or ordinary-function construction; Generator parameter/
result transfer; repeated consumption; abandonment; libraries; and external
boundaries SHALL remain unsupported. The lowering SHALL introduce no semantic
Generator object or state allocation, generic Optional/product dispatcher,
callback, indirect call, unwind dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library calling
convention or Generator/Optional/product ABI, or native-ABI revision. Optional,
product, Int, and String allocation, equality, display, and Linux syscalls
SHALL remain wholly owned by Topal. Future compiled-library metadata SHALL
encode recursive classifiers, independent directions, nominal alternatives,
product arity/order, field identities, equality and ordered yield/action/
resume/final provenance, allocation-failure effects, declaration/construction
sites, captures/effects, ownership/consumption/close state, native-
representation identity, and target adapters rather than expose private
tokens, debug slots, object layouts, or checked-program nodes.

### TOPAL-COMPILER-GENERATOR-NESTED-RESULT-001 — Recursive Result product directions

The compiler SHALL admit a root custom generator with one named
`Result ((Int, String), lang arithmetic ArithmeticErrorCode)` initial parameter
and the same yield and result directions with Unit resumption. Its body SHALL
consist only of one discarded `yield initial` followed by `(8, "done")`. The
shared regression SHALL start the generator with exact `(7, "item")`; the fresh
result SHALL be bound and consumed exactly once by a foreach action consisting
only of the discarded `candidate = (7, "item")` for its named yielded
parameter. Each product SHALL satisfy its explicit Result contract as an
implicit success value.

The checked program SHALL retain the nominal Result and arithmetic-error
vocabulary identities, success alternative, positional-product arity and
order, and Int and String field identities separately in all three Generator
directions. It SHALL also retain the initial-parameter yield and suspension,
success-gated field equality action, Unit resumption, distinct final product,
declaration provenance, and ownership edge. Application SHALL evaluate both
initial payload fields once in source order and construct one successful
Result. Traversal SHALL pass that same immutable value to the action, construct
the action success operand once, inspect both success/error tags before
observing success payloads, compare payload fields in source order, resume with
Unit, and only then construct the successful `(8, "done")`. This order SHALL
hold at LLVM O0 without relying on folding, inlining, or dead-code elimination.

On Linux x86-64, Result SHALL retain its existing Topal-owned tagged pointer
header. Each admitted successful product payload SHALL use a Topal-owned
aligned 16-byte allocation containing the existing Int and String pointers in
source order. The root-local Generator SHALL remain a compiler-private `i32`
ownership token. LLVM SHALL derive placement, alignment, and call lowering
from the target triple and data layout; the compiler SHALL hard-code no AMD64
register convention. Two aligned debug-only Result pointer shadows SHALL
preserve the yielded action value and captured initial lifetime. DWARF and GDB
SHALL expose the complete recursive Generator classifier and value, the Result
product classifier, ordered payload fields, both yielded/captured `(7,
"item")` values, ordered yield/action/resumption/final source locations, and
the Topal entry frame.

An Error value, another Result success type, product arity, field order, input,
action, final, direction, yield, or value; multiple yields; additional body
statements; close handling; nested or ordinary-function construction;
Generator parameter/result transfer; repeated consumption; abandonment;
libraries; and external boundaries SHALL remain unsupported. The lowering
SHALL introduce no semantic Generator object or state allocation, generic
Result/product dispatcher, callback, indirect call, unwind dependency, C/C++
runtime, other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator/Result/product ABI, or native-
ABI revision. Result, product, Int, and String allocation, equality, display,
and Linux syscalls SHALL remain wholly owned by Topal. Future compiled-library
metadata SHALL encode recursive classifiers, independent directions, nominal
alternatives/error vocabularies, product arity/order, field identities,
success/error evidence, equality and ordered yield/action/resume/final
provenance, allocation-failure effects, declaration/construction sites,
captures/effects, ownership/consumption/close state, native-representation
identity, and target adapters rather than expose private tokens, debug slots,
object layouts, or checked-program nodes.

### TOPAL-COMPILER-GENERATOR-FINAL-DECISION-001 — Post-resume final Boolean decision

The compiler SHALL admit a root custom generator with one named Boolean
initial parameter and directions `Generator Boolean Unit String`. Its body
SHALL consist only of one discarded `yield initial` followed by a complete
decision on `initial` whose `true` action is the exact String `"accepted"` and
whose `otherwise` action is the exact String `"rejected"`. The shared
regression SHALL start the generator with `true`; the fresh result SHALL be
bound and consumed exactly once by a foreach action consisting only of the
discarded `not value` for its named yielded parameter.

The checked program SHALL retain the independent Boolean yield and String
final directions, the initial-parameter suspension, action, Unit resumption,
decision subject and ordered alternatives/actions, declaration provenance, and
ownership edge. Application SHALL evaluate the initial Boolean once. Traversal
SHALL pass that same value to the action, resume with Unit, evaluate the
captured initial as the decision subject once, execute exactly the selected
String action, and return that String as the final/root value. LLVM O0 SHALL
retain a direct conditional branch, distinct true/false String blocks, and a
typed join without relying on folding, inlining, or dead-code elimination.

On Linux x86-64, the Boolean SHALL remain a private `i1`, each selected String
SHALL use the existing Topal-owned immutable descriptor, and the root-local
Generator SHALL remain a compiler-private `i32` ownership token. LLVM SHALL
derive placement, alignment, branch lowering, and calls from the target triple
and data layout; the compiler SHALL hard-code no AMD64 register convention. An
aligned debug-only Boolean shadow SHALL preserve the yielded action value.
DWARF and GDB SHALL expose the complete Generator classifier and value, yielded
and captured Boolean values, ordered yield/action/decision source locations,
and the Topal entry frame.

Another input classifier, yield, action, decision subject, matcher order,
String action, direction, value, or final expression; multiple yields; additional body
statements; close handling; nested or ordinary-function construction;
Generator parameter/result transfer; repeated consumption; abandonment;
libraries; and external boundaries SHALL remain unsupported. The lowering
SHALL introduce no semantic Generator object or state allocation, generic
decision or Generator dispatcher, callback, indirect call, unwind dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library calling convention or Generator/String ABI, or
native-ABI revision. Boolean decisions, String construction/display, and Linux
syscalls SHALL remain wholly owned by Topal. Future compiled-library metadata
SHALL encode independent directions, ordered decision matchers/actions,
subject/final provenance, declaration/construction sites, captures/effects,
ownership/consumption/close state, native-representation identity, and target
adapters rather than expose private tokens, debug slots, LLVM blocks, object
layouts, or checked-program nodes.

### TOPAL-COMPILER-GENERATOR-NESTED-NONE-001 — Absent recursive Optional directions

The compiler SHALL admit a root custom generator with one named
`Optional (Int, String)` initial parameter and directions
`Generator Optional (Int, String) Unit Optional (Int, String)`. Its body SHALL
consist only of one discarded `yield initial` followed by
`None (Int, String)`. The shared regression SHALL start the generator with
exact `None (Int, String)`; the fresh result SHALL be bound and consumed exactly
once by a foreach action consisting only of the discarded
`candidate = (None (Int, String))` for its named yielded parameter.

The checked program SHALL retain the nominal Optional identity, None
alternative, positional-product arity and order, and Int and String field
identities separately in all three Generator directions despite the absence of
a payload. It SHALL also retain the initial-parameter yield and suspension,
tag-equality action, Unit resumption, distinct final None construction,
declaration provenance, and ownership edge. Application SHALL construct the
initial absent Optional once. Traversal SHALL pass that same immutable value to
the action, construct the absent action operand once, compare tags before any
payload observation, resume with Unit, and only then construct the final absent
Optional. This order SHALL hold at LLVM O0 without relying on folding, inlining,
or dead-code elimination.

On Linux x86-64, each admitted None SHALL use the existing Topal-owned Optional
tagged pointer header with no product-payload allocation. The root-local
Generator SHALL remain a compiler-private `i32` ownership token. LLVM SHALL
derive placement, alignment, and call lowering from the target triple and data
layout; the compiler SHALL hard-code no AMD64 register convention. Two aligned
debug-only Optional pointer shadows SHALL preserve the yielded action value and
captured initial lifetime. DWARF and GDB SHALL expose the complete recursive
Generator classifier and value, the Optional product classifier and ordered
payload field types, both yielded/captured None values, ordered yield/action/
resumption/final source locations, and the Topal entry frame.

A Some value, mixed Some/None graph, another Optional payload, product arity,
field order, input, action, final, direction, yield, or value; multiple yields;
additional body statements; close handling; nested or ordinary-function
construction; Generator parameter/result transfer; repeated consumption;
abandonment; libraries; and external boundaries SHALL remain unsupported. The
lowering SHALL introduce no semantic Generator object or state allocation,
generic Optional/product dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator/Optional/
product ABI, or native-ABI revision. Optional tag construction/equality,
display, and Linux syscalls SHALL remain wholly owned by Topal. Future
compiled-library metadata SHALL encode recursive classifiers, independent
directions, nominal alternatives, product arity/order and field identities,
absence evidence, equality and ordered yield/action/resume/final provenance,
allocation-failure effects, declaration/construction sites, captures/effects,
ownership/consumption/close state, native-representation identity, and target
adapters rather than expose private tokens, debug slots, object layouts, or
checked-program nodes.

### TOPAL-COMPILER-GENERATOR-PRODUCT-001 — Exact positional-product generator directions

The compiler SHALL admit a root custom generator with one named `(Int, String)`
initial parameter and directions
`Generator (Int, String) Unit (Int, String)`. Its body SHALL consist only of
one discarded `yield initial` followed by `(8, "done")` as its final expression.
The shared regression starts the generator with `(7, "item")`; the fresh result
SHALL be bound and consumed exactly once by a foreach action consisting only of
the discarded `value = (7, "item")` for its named yielded parameter.

The checked program SHALL retain positional-product arity, source field order,
the Int and String field classifiers separately in all three Generator
directions, the initial-parameter yield and suspension, field-wise equality
action, Unit resumption, distinct final product, declaration provenance, and
ownership edge. Application SHALL evaluate both initial fields once from left
to right. Traversal SHALL pass that same immutable product to the action,
compare its Int and String fields in order, resume with Unit, and only then
materialize `(8, "done")` as the final product. This order SHALL hold with LLVM
optimization disabled and SHALL NOT depend on folding, inlining, or dead-code
elimination.

On Linux x86-64, the product SHALL use the existing compiler-private LLVM
aggregate of Topal-owned Int and String pointers, and the root-local Generator
SHALL remain a compiler-private `i32` ownership token. LLVM SHALL derive
aggregate placement, alignment, and call lowering from the target triple and
data layout; the compiler SHALL hard-code no AMD64 register convention. Two
aligned debug-only product shadows and four lifetime/source anchor stores SHALL
keep the yielded action value and captured initial inspectable. DWARF and GDB
SHALL expose the complete
`Generator (Int, String) Unit (Int, String)` classifier and value, ordered `_0`
and `_1` product members, the yielded `(7, "item")`, captured initial
`(7, "item")`, ordered yield/action/resumption/final source locations, and the
Topal entry frame.

Another product arity, field classifier, order, literal, or direction; literal
or multiple yields; a final expression or action other than the exact products
above; labeled products; additional body statements; close handling; nested or
ordinary-function construction; Generator parameter/result transfer; repeated
consumption; abandonment; libraries; and external boundaries SHALL remain
unsupported. The lowering SHALL introduce no semantic Generator object or
state allocation, dispatcher, callback, indirect call, unwind dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library calling convention or Generator/product ABI, or
native-ABI revision. Product equality/display, Int/String storage, and Linux
syscalls SHALL remain wholly owned by Topal. Future compiled-library metadata
SHALL encode independent initial and direction classifiers, positional-product
arity and ordered field identities, field equality, ordered yield/action/
resume/final provenance, allocation-failure effects, declaration and
construction sites, captures/effects, ownership/consumption/close state,
native-representation identity, and target adapters rather than expose the
private token, debug slots, aggregate/pointer layouts, or checked-program node
layout.

### TOPAL-COMPILER-GENERATOR-ENUM-001 — Exact nominal Enum generator directions

The compiler SHALL admit the exact prior declaration
`Choice is Enum (First, Second)` followed by a root custom generator with one
named `Choice` initial parameter and directions
`Generator Choice Unit Choice`. Its body SHALL consist only of one discarded
`yield initial` followed by `Second` as its final expression. The shared
regression starts the generator with `First`; the fresh result SHALL be bound
and consumed exactly once by a foreach action consisting only of the discarded
`choice = First` for its named yielded parameter.

The checked program SHALL retain the nominal Choice identity, ordered First and
Second alternatives and private tags separately from all three Generator
directions, the initial-parameter yield and suspension, exact equality action,
Unit resumption, distinct final alternative, declaration provenance, and
ownership edge. Application SHALL evaluate its initial expression exactly once.
Traversal SHALL pass that same immutable Choice value to the action, compare it
with First once, resume with Unit, and only then produce Second as the final
Choice value. This order SHALL hold with LLVM optimization disabled and SHALL
NOT depend on folding, inlining, or dead-code elimination.

On Linux x86-64, Choice SHALL retain its existing compiler-private `i32` tag
representation, and the root-local Generator SHALL remain a compiler-private
`i32` ownership token. LLVM SHALL select placement and call lowering from the
target triple and data layout; the compiler SHALL hard-code no AMD64 register
convention. Two aligned debug-only `i32` shadow slots and four lifetime/source
anchor stores SHALL keep the yielded action value and captured initial
inspectable even when LLVM folds the constant equality and final tag selection.
DWARF and GDB SHALL expose the complete `Generator Choice Unit Choice`
classifier and value, the Choice alternatives, yielded First, captured initial
First, ordered yield/action/resumption/final source locations, and the Topal
entry frame.

Another Enum declaration, alternatives, order, or direction; literal or
multiple yields; a final expression or action other than the exact alternatives
above; additional body statements; close handling; nested or ordinary-function
construction; Generator parameter/result transfer; repeated consumption;
abandonment; libraries; and external boundaries SHALL remain unsupported. The
lowering SHALL introduce no semantic Generator object or state allocation,
dispatcher, callback, indirect call, unwind dependency, C/C++ runtime, other-
language standard library, needed library, dynamic relocation, public/library
calling convention or Generator ABI, or native-ABI revision. Enum comparison,
display, and Linux syscalls SHALL remain wholly owned by Topal. Future compiled-
library metadata SHALL encode independent initial and direction classifiers,
nominal Enum identity, ordered alternative identities and tags, equality
operands, ordered yield/action/resume/final provenance, declaration/construction
sites, captures/effects, ownership/consumption/close state, native-
representation identity, and target adapters rather than expose the private
token, debug slots, tag layout, or checked-program node layout.

### TOPAL-COMPILER-GENERATOR-NAT-001 — Exact Nat generator directions

The compiler SHALL admit a root custom generator with one named `Nat` initial
parameter and directions `Generator Nat Unit Nat` whose body consists only of
one discarded `yield initial` followed by `initial + 1` as its final expression.
One currently admitted `Nat` expression SHALL start the generator; the shared
regression uses the statically proven construction `Nat 7`. The fresh result
SHALL be bound and consumed exactly once by a foreach action consisting only of
the discarded `value + 1` for its named yielded parameter.

The checked program SHALL retain the Nat refinement identity separately from
its underlying exact Int representation and from all three Generator
directions, the proof-bearing input construction, initial-parameter yield and
suspension, exact action and final additions, Unit resumption, and ownership
edge. Application SHALL evaluate its initial expression exactly once. For the
shared closed input, construction SHALL preserve the nonnegative proof without
a runtime validation call. Traversal SHALL pass that same immutable Nat value
to the action, add exact one once, resume with Unit, and only then add exact one
to the captured initial for the final Nat value `8`. This order SHALL hold with
LLVM optimization disabled and SHALL NOT depend on folding, inlining, or dead-
code elimination.

On Linux x86-64, Nat SHALL retain its existing compiler-private representation
as an aligned pointer to Topal-owned arbitrary-precision Int storage; the
nonnegative constraint SHALL NOT introduce a native unsigned-integer ABI. The
root-local Generator SHALL remain a compiler-private `i32` ownership token.
LLVM SHALL select placement and call lowering from the target triple and data
layout; the compiler SHALL hard-code no AMD64 register convention. Two aligned
debug-only pointer shadows SHALL preserve the yielded action value and captured
initial lifetime. DWARF and GDB SHALL expose the complete
`Generator Nat Unit Nat` classifier and value, the yielded Nat `7`, the
captured initial Nat `7`, ordered yield/action/resumption/final source
locations, and the Topal entry frame.

Literal or multiple yields; a final expression or action other than the exact
increments; additional body statements; another input, yield, resume, or final
classifier; general Nat arithmetic outside this exact graph; close handling;
nested or ordinary-function construction; Generator parameter/result transfer;
repeated consumption; abandonment; libraries; and external boundaries SHALL
remain unsupported. The lowering SHALL introduce no semantic Generator object
or state allocation, dispatcher, callback, indirect call, unwind dependency,
C/C++ runtime, other-language standard library, needed library, dynamic
relocation, public/library calling convention or Generator ABI, unsigned
machine arithmetic, or native-ABI revision. Exact integer storage, allocation,
addition, display, and Linux syscalls SHALL remain wholly owned by Topal. Future
compiled-library metadata SHALL encode independent initial and direction
classifiers, Nat refinement/proof and underlying Int identities, validation
provenance, exact addition operands, ordered yield/action/resume/final
provenance, allocation-failure effects, declaration/construction sites,
captures/effects, ownership/consumption/close state, native-representation
identity, and target adapters rather than expose the private token, debug slots,
Int object layout, or checked-program node layout.

### TOPAL-COMPILER-GENERATOR-RANGE-001 — Exact Range generator directions

The compiler SHALL admit a root custom generator with one named `Range Int`
initial parameter and directions `Generator Range Int Unit Range Int` whose
body consists only of one discarded `yield initial` followed by
`initial and (5 ..= 15)` as its final expression. One currently admitted
`Range Int` expression SHALL start the generator; the shared regression uses
`0 ..= 10`. The fresh result SHALL be bound and consumed exactly once by a
foreach action consisting only of the discarded membership `5 in interval`
for its named yielded parameter.

The checked program SHALL retain the Range classifier and nominal Int endpoint
classifier separately from all three Generator directions, the initial-
parameter yield and suspension, exact membership action, Unit resumption,
distinct final inclusive-bound construction and intersection, and ownership
edge. Application SHALL evaluate and construct its initial expression exactly
once. Traversal SHALL pass that same immutable Range value to the action,
evaluate membership once, resume the generator with Unit, and only then
construct `5 ..= 15` and intersect it with the initial value. The resulting
inclusive range SHALL be `5 ..= 10` for the shared source. This order SHALL
hold with LLVM optimization disabled and SHALL NOT depend on folding, inlining,
or dead-code elimination.

On Linux x86-64, the existing compiler-private Range representation SHALL
remain an aligned pointer to Topal-owned finite storage containing Int endpoint
pointers and inclusion flags. The root-local Generator SHALL remain a compiler-
private `i32` ownership token. LLVM SHALL select placement and call lowering
from the target triple and data layout; the compiler SHALL hard-code no AMD64
register convention. Two aligned debug-only pointer shadows SHALL preserve the
yielded action value and captured initial lifetime. DWARF and GDB SHALL expose
the complete `Generator Range Int Unit Range Int` classifier and value, the
yielded `0 ..= 10`, the captured initial `0 ..= 10`, ordered yield/action/
resumption/final source locations, and the Topal entry frame.

Literal or multiple yields; a final expression other than the exact
intersection; additional body statements; another endpoint, action, input,
yield, resume, or final classifier; close handling; nested or ordinary-function
construction; Generator parameter/result transfer; repeated consumption;
abandonment; libraries; and external boundaries SHALL remain unsupported. The
lowering SHALL introduce no semantic Generator object or state allocation,
dispatcher, callback, indirect call, unwind dependency, C/C++ runtime, other-
language standard library, needed library, dynamic relocation, public/library
calling convention or Generator ABI, or native-ABI revision. Range/Int storage,
allocation, membership, intersection, display, and Linux syscalls SHALL remain
wholly owned by Topal. Future compiled-library metadata SHALL encode independent
initial and direction classifiers, the nominal endpoint identity, exact bounds,
inclusivity, membership and intersection operands, ordered yield/action/resume/
final provenance, allocation-failure effects, declaration/construction sites,
captures/effects, ownership/consumption/close state, native-representation
identity, and target adapters rather than expose the private token, debug slots,
Range/Int object layouts, or checked-program node layout.

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

### TOPAL-COMPILER-ROOT-NAMESPACE-001 — Direct executable-root qualification

In the admitted single-source application subset, `root` in value position
SHALL denote the current executable root namespace and canonically display as
`<namespace root>` without copying, flattening, or executing its declarations.
A qualified ordinary or static function path beginning `root member` SHALL
resolve `member` only from
the collected root declaration set before applying remaining operands with the
ordinary checked overload and evaluation rules. A lexically shadowing local
binding with the same member name SHALL NOT intercept that qualified selection.

The root value MAY use a sealed zero-data compiler-private tag when carried in
an expression-local positional product. Qualified calls SHALL lower directly to
the selected private LLVM function and retain its source subprogram, parameter,
location, and call frame in DWARF/GDB at O0. No runtime namespace lookup, Scope
object allocation, foreign dependency, C/C++ runtime, other-language standard
library, public tag ABI, or native ABI revision is permitted.

Qualified namespace-alias lookup, classified Scope bindings, data-member
lookup, `use`, published interfaces, generators, package loading, and
compiled-library resolution remain outside this increment and SHALL be rejected
rather than reinterpreted as direct-root function qualification.

### TOPAL-COMPILER-NAMESPACE-USE-001 — Static root namespace use

At source root, `use` applied to the live `root` Scope or an already retained
root-namespace alias SHALL produce that same namespace value for optional
binding. The binding SHALL retain the declaration snapshot visible at that
statement under the existing namespace function, data, overload, classifier,
and alias-chain rules. It SHALL NOT flatten members into the current lexical
scope. Applying `use` to a non-Scope value SHALL produce
`E-USE-NON-NAMESPACE` before code generation.

Qualified application through the resulting value SHALL reuse the existing
checked namespace snapshot and direct private function/data lowering. The
`use` operation itself SHALL have no LLVM instruction or runtime state; an
observed binding MAY use the existing sealed private Scope tag and SHALL retain
its source name and Scope type in DWARF/GDB. No namespace lookup table,
allocation, indirect dispatch, foreign dependency, C/C++ runtime,
other-language standard library, public Scope ABI, or native ABI revision is
permitted.

Multi-component and non-root published paths, nested/non-root namespaces,
generator members, function-body `use`, packages, source or compiled libraries,
and public interface metadata remain outside this increment and SHALL be
rejected rather than resolved from process state or the host filesystem.

### TOPAL-COMPILER-NAMESPACE-FUNCTION-ALIAS-001 — Static function namespace aliases

At source root, binding the live `root` Scope value or an already admitted
namespace alias SHALL produce an immutable compiler-known namespace snapshot.
The snapshot SHALL retain the namespace identity and every ordinary and static
function declaration visible at the binding statement, including each
source-ordered overload set. A later declaration SHALL remain visible through a
later live `root` selection but SHALL NOT enter an earlier alias. A finite alias
chain SHALL retain the original captured function set, and an explicit `Scope`
classifier SHALL preserve rather than erase that information.

An application `alias member operands` SHALL resolve `member` only in the
captured function set before applying the ordinary checked overload rules. A
same-named caller binding SHALL NOT intercept or join that selection. The
selected call SHALL lower to the existing private direct LLVM function call.
The alias value MAY reuse the sealed private root tag for canonical display and
DWARF/GDB observation; captured declarations SHALL remain frontend metadata and
SHALL NOT cause a runtime lookup table, function pointer dispatch, Scope object
allocation, foreign dependency, C/C++ runtime, other-language standard library,
public ABI, or native ABI revision.

Root and alias data members, generator members, aliases outside source root,
general Scope function boundaries, `use`, packages, and source or compiled
libraries remain outside this increment and SHALL be rejected.

### TOPAL-COMPILER-NAMESPACE-DATA-001 — Stable root data snapshots

Within source-root executable blocks, each admitted immutable binding SHALL
receive a distinct compiler storage identity in addition to its source/debug
name. After its initializer completes, the binding SHALL become visible as a
data member of the live `root` namespace. A root alias SHALL capture exactly the
data members visible at that alias binding, and an alias chain SHALL retain that
same captured set. A later binding SHALL remain visible through a later live
`root` selection but SHALL NOT enter an earlier alias. These rules apply equally
to a source-root binding prefixed by `pub`; publication SHALL NOT imply a native
export in this single-source increment.

Direct `root member` and `alias member` data selection SHALL reference the
original immutable storage identity. It SHALL NOT re-evaluate or copy the
initializer, and a same-named lexical binding SHALL NOT intercept the qualified
reference. Source names SHALL remain unchanged in DWARF even when internal
storage identities differ. The implementation SHALL require no runtime
namespace lookup, Scope allocation, foreign dependency, C/C++ runtime,
other-language standard library, public ABI, or native ABI revision.

Direct `root member` data access from a compiled function body, Scope
results/escape, nested qualified Scope members, generators, `use`, packages,
and source or compiled libraries remain outside this increment and SHALL be
rejected until a storage/interface representation valid beyond the source entry
frame exists.

### TOPAL-COMPILER-NAMESPACE-BOUNDARY-001 — Specialized Scope parameters

When an admitted single-source call supplies the live `root` value, a known
root alias, or an already-specialized Scope parameter to an ordinary parameter
classified as `Scope`, checking SHALL specialize the callee with that namespace
identity and its exact captured declaration snapshot. Qualified function
selection through the parameter SHALL retain the snapshot overloads and lower
to direct private calls. Each data member with an admitted private
representation SHALL retain the original already-evaluated value and
source-member identity across the call. Forwarding the parameter to another
admitted Scope boundary SHALL preserve the same facts. A missing member and any
selected member without a valid private representation SHALL be rejected before
code generation.

The explicit Scope value MAY remain the sealed compiler-private tag. The
compiler MAY closure-convert a non-discarded parameter's finite represented data
snapshot into exact hidden LLVM parameters, including unused members needed to
make subsequent forwarding independent of caller storage. Definitions and
calls SHALL agree on one exact private signature and leave physical AMD64
aggregate and register classification to LLVM. Initializers SHALL NOT be
re-executed, and LLVM values
SHALL NOT acquire source-level copy or identity semantics merely by crossing the
boundary. DWARF/GDB SHALL expose the explicit Scope parameter and any material
hidden data arguments without presenting a runtime lookup object.

This specialization SHALL NOT define a public Scope or environment ABI and
SHALL require no namespace table, indirect dispatch, allocation, foreign
dependency, C/C++ runtime, other-language standard library, or native ABI
revision. Scope results or escape, a live `root` argument formed inside a
compiled function, nested or non-root namespaces, generator members, `use`,
packages, and compiled-library Scope environments remain deferred.

### TOPAL-COMPILER-NAMED-FUNCTION-VALUE-001 — Retained named function values

Resolving an already-visible ordinary or static root function in value position
SHALL produce a checked Function value retaining the original source name,
complete visible declaration set, source overload order, and staticness. Binding
that value, including through a finite chain of bindings, SHALL preserve those
facts. Applying the bound name SHALL select only from the retained declarations
under the ordinary argument, overload, static-context, termination, and result
rules; it SHALL NOT restart lookup using the binding name or combine candidates
from a caller lexical scope.

The compiler MAY assign each root function name a deterministic module-private
integer tag for canonical `<fn name>` display and truthful Function DWARF/GDB
observation. That tag SHALL NOT determine application: the retained compile-time
declaration identity SHALL produce a direct private LLVM call to the selected
specialization. No function-pointer dispatch, closure allocation, Function
runtime, foreign dependency, C/C++ runtime, other-language standard library,
public callable ABI, or native ABI revision is permitted.

Symbolic callable values, anonymous functions and captures, Function parameters
or results, namespace selection of a function-valued data binding, and published
callable interfaces remain outside this increment and SHALL be rejected.

### TOPAL-COMPILER-SYMBOLIC-CALLABLE-VALUE-001 — Direct symbolic Function values

The symbolic callables `+`, `-`, and `<=>` in value position SHALL produce
Function values retaining their exact callable identity. A binding or finite
binding chain MAY classify that value as `Function` without erasing the
identity. Applying bound `+` or `<=>` SHALL require one two-field positional
product and apply the existing checked binary-operation rules. Bound `-` SHALL
accept either that product for subtraction or one direct exact-numeric operand
for negation. Invalid arity SHALL produce a no-applicable-overload diagnostic.

The deterministic private Function observation table SHALL include the three
canonical spellings, and display/DWARF/GDB SHALL report those spellings. Each
application SHALL lower directly to the corresponding existing LLVM operation
or Topal-owned runtime primitive; the Function tag SHALL NOT dispatch it. No
function pointer, indirect call, Function runtime, closure allocation, foreign
dependency, C/C++ runtime, other-language standard library, public callable ABI,
or native ABI revision is permitted.

Other symbolic callable values, Function parameters/results, anonymous or
capturing functions, and published callable interfaces remain outside this
increment and SHALL be rejected.

### TOPAL-COMPILER-FUNCTION-PARAMETER-001 — Specialized private Function inputs

An ordinary or static function parameter classified as `Function` SHALL accept
an admitted named or symbolic Function value. At each call site, the checked
compiler SHALL retain the argument's callable metadata while specializing the
callee body. Application through the parameter SHALL use only that retained
identity and SHALL lower to the same direct function call or operation as a
non-parameter application. A bound Function argument SHALL retain its metadata
when passed; multiple callable identities MAY produce separate private callee
specializations.

The exact private LLVM signature SHALL carry the deterministic i32 Function
observation tag in the source parameter position. The tag SHALL preserve value
and debugging semantics but SHALL NOT dispatch application. Because static
specialization can otherwise make the machine parameter computationally dead,
O0 code generation SHALL retain a target-aligned debug-only stack shadow so GDB
can inspect the source Function parameter and call frame. No function pointer,
indirect call, closure allocation, Function runtime, foreign dependency, C/C++
runtime, other-language standard library, public callable ABI, or native ABI
revision is permitted.

Function results, Function values embedded in aggregate boundaries, anonymous
functions/captures, remaining symbolic callables, and published callable
interfaces remain outside this increment and SHALL be rejected.

### TOPAL-COMPILER-ANONYMOUS-DIRECT-001 — Private direct anonymous functions

An inferred anonymous function containing only binding parameter patterns and
no lexical data captures SHALL be retained as a checked Function value with its
source body, parameter arity, and construction identity. It MAY be bound before
application or supplied directly to a private parameter classified as
`Function`. A unary application SHALL infer its one parameter classifier from
the direct operand. A multi-parameter application SHALL require one positional
product of the same arity, evaluate its fields left-to-right, and infer the
corresponding parameter classifiers in source order. The body result classifier
SHALL be inferred after those parameter bindings are established. Mixed
symbolic applications in the body SHALL retain the language's left-to-right,
no-hidden-precedence grouping.

Each admitted application SHALL produce a private specialized LLVM function
with an exact `fastcc` signature and a direct call. The compiler MAY use a
deterministic module-private Function tag for canonical `<anonymous fn/N>`
display and truthful DWARF/GDB bindings, but the tag SHALL NOT dispatch the
call. DWARF SHALL expose the anonymous source frame and inferred source
parameters. No function pointer, indirect call, closure allocation, closure or
Function runtime, foreign dependency, C/C++ runtime, other-language standard
library, public callable ABI, or native ABI revision is permitted.

Lexical data captures, anonymous product parameter patterns, escaping closure
storage, Function results or aggregate Function boundaries, and published
callable interfaces remain outside this increment and SHALL be rejected rather
than referencing storage from another call frame.

### TOPAL-COMPILER-NESTED-FUNCTION-001 — Private direct nested lexical functions

Within the direct statements of an admitted ordinary, non-static function body,
an unpublished ordinary nested function declaration SHALL become visible at
its declaration point in that invocation scope. Direct application of its name
SHALL use its declared parameter and result classifiers and SHALL execute in a
fresh invocation. Each admitted immutable lexical data binding visible when the
nested function is declared MAY be retained as capture metadata when it has a
complete private compiler representation; an explicit nested parameter of the
same name SHALL shadow that outer binding. The original captured value SHALL be
passed without re-evaluating its initializer or reading another call frame's
storage.

Each direct application SHALL lower to a compiler-private specialized function
and direct `fastcc` call. The exact LLVM signature SHALL list ordinary source
parameters first and append exact typed capture parameters in a deterministic
order. Definitions and calls SHALL agree, while LLVM owns physical AMD64
register, stack, and aggregate classification. The compiler MAY over-capture
the finite represented lexical environment so long as this does not alter
source evaluation or identity. DWARF/GDB SHALL expose the nested source frame,
ordinary parameters, and material capture parameters using their source names,
classifiers, and values.

The nested function name SHALL remain non-escaping compiler metadata: using it
as an ordinary Function value, returning it, storing it in an aggregate, or
passing it through a Function boundary SHALL be rejected. The lowering SHALL
require no caller-frame reference, environment object, closure allocation or
runtime, function pointer, indirect call, foreign dependency, C/C++ runtime,
other-language standard library, public callable ABI, or native ABI revision.

Declarations inside nested lexical/decision blocks and published, static,
measured, constrained, or effectful nested functions; nested overload sets,
recursion, sibling calls, collisions with visible or active named callables,
anonymous captures, Scope, Function, Constraint, refined-evidence, or
defining-context captures; escaping closures; and public/library closure
metadata remain outside this increment and SHALL be rejected rather than
receiving a provisional closure representation.

### TOPAL-COMPILER-PACKAGED-OPERAND-001 — Closed scalar packaged operand

The compiler SHALL admit a function with exactly one syntactic operand package
whose fields all use admitted scalar classifiers. A positional product SHALL
supply every field in declaration order. A labeled product SHALL supply a
declaration-order prefix containing every nondefaulted field; omitted trailing
fields SHALL each have a default expression that the checked model proves
closed. Supplied expressions SHALL be evaluated once in source/declaration
order, followed by each omitted default once in declaration order, and every
value SHALL undergo the field's ordinary classifier adaptation before entry.
Unknown labels, missing required fields, and classifier mismatches SHALL remain
ordinary no-applicable-overload failures.

The checked frontend SHALL normalize the package to its source fields and emit
an exact flattened private `fastcc` signature and direct call. LLVM SHALL own
the physical register/stack lowering of that signature. DWARF/GDB SHALL expose
the source field names, classifiers, values, and call frame. The lowering SHALL
NOT use `byval`, `sret`, `inalloca`, or `preallocated`, and SHALL require no
package runtime, allocation, foreign dependency, C/C++ runtime, other-language
standard library, public aggregate ABI, or native ABI revision.

Multiple packaged operands, packages mixed with unpackaged operands,
non-scalar or nested package fields, non-prefix/reordered labeled packages,
opaque package values, and defaults that depend on invocation or captured
bindings remain outside this increment and SHALL be rejected rather than
changing evaluation order or choosing a public memory ABI.

### TOPAL-COMPILER-CONTEXT-CAPTURE-001 — Private defining-context capture

For a root function called directly from the source entry frame, each `@ member`
reference to an admitted scalar root binding declared before that function SHALL
resolve only that immutable defining-context binding. A same-named caller or
lexical binding SHALL NOT intercept the selection, and a member declared after
the function SHALL remain unavailable. The original root initializer SHALL be
evaluated once; capture lowering SHALL pass its resulting compiler value rather
than re-evaluate the initializer or consult process-global state.

The checked frontend SHALL append each referenced member in root declaration
order as an explicit private capture parameter and direct call argument. Its
LLVM type SHALL be the existing exact private type for the member classifier,
and LLVM SHALL own physical target placement under `fastcc`. DWARF/GDB SHALL
expose the parameter as `@ member`, with its source classifier and value, inside
the ordinary function frame. No global variable, context table or lookup,
function pointer, indirect call, closure allocation/runtime, foreign dependency,
C/C++ runtime, other-language standard library, public closure ABI, or native
ABI revision is permitted.

Selection outside a function SHALL remain a context-selection error. Aggregate,
Scope, or Function members, calls requiring capture forwarding between compiled
functions, anonymous captures, escaping functions, qualified `root member`
access from functions, and public/library context environments remain outside
this increment and SHALL be rejected.

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

### TOPAL-COMPILER-RECURSION-OVERLOAD-IDENTITY-001 — Overload-specific call graph identity

The compiler SHALL key recursion and active-call detection by the complete
selected source input header and staticness, not by the function name or LLVM
machine prototype alone. A call from one active overload to a same-named
overload with a different input header SHALL be an ordinary call edge and SHALL
NOT require recursion evidence unless its own call graph returns to an active
identity.

Each selected overload SHALL retain a distinct private symbol, checked source
signature, and DWARF subprogram even when two source classifiers use the same
private machine carrier. Callee-before-caller emission SHALL remain valid for
an acyclic cross-overload edge. This identity distinction SHALL add no runtime
dispatch, type tag, foreign dependency, standard library, or native ABI
revision.

### TOPAL-COMPILER-RECURSION-INT-MUTUAL-001 — Proven mutual Int recursion

The compiler SHALL admit a back-edge across two or more unary `Int` overloads
only when the shared proofs establish a complete cycle under
`TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001` or
`TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001`. Every active member SHALL
carry the same direction-specific proof, name the next active member, and close
the cycle in call order. An isolated candidate, a missing or differently typed
member, a mixed-direction cycle, or any invalid edge SHALL remain rejected.
Every repeated next-member call SHALL independently satisfy
`TOPAL-FUNCTION-RECURSION-ALL-CALLS-001`.

Each instantiated member SHALL retain one exact private symbol and prototype;
the closing edge SHALL target the already reserved active member. Correctness
at O0 SHALL NOT depend on tail-call conversion, inlining, or another LLVM
optimization, and no cycle member SHALL be marked `norecurse`. DWARF/GDB SHALL
retain the distinct source members and recursive frames. This proof SHALL add
no dispatcher, hidden parameter, foreign dependency, standard library, or
native ABI revision.

### TOPAL-COMPILER-RECURSION-NAT-MUTUAL-001 — Proven mutual Nat recursion

The compiler SHALL admit a back-edge across two or more unary `Nat` overloads
only when the shared proofs establish a complete cycle under
`TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001` or
`TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001`. Every active member SHALL
carry the same direction-specific proof, name the next active member, and close
the cycle in call order. The decreasing proof SHALL establish that every
literal decrement remains nonnegative from that member's recursive region.
Unsafe overshoot, incomplete or mixed-direction cycles, and invalid edges SHALL
remain rejected.

Only the proven unary next-member argument MAY regain `Nat` evidence after
arithmetic has exposed the unchanged `Int` carrier. That evidence conversion
SHALL emit no dynamic Nat validation. Each member SHALL retain one exact private
prototype, `noinline` behavior at O0, no false `norecurse` attribute, and a
distinct source-level Nat parameter and frame in DWARF/GDB. The proof SHALL add
no unsigned representation, dispatcher, hidden state, foreign dependency,
standard library, or native ABI revision.

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

### TOPAL-COMPILER-SUM-001 — Sealed nominal sum lowering

Each admitted root-scope labeled `Union` or positional `Variant` SHALL retain
its distinct nominal identity, declaration-ordered alternatives, and exact
payload classifiers through checking. Construction SHALL evaluate and classify
the selected complete payload once. A sum decision SHALL evaluate its subject
once, select only the active alternative, bind its complete payload only in the
selected action, and enforce `TOPAL-DECISION-UNION-001` completeness and
matcher validity. Display SHALL agree with the shared interpreter.

The private native representation MAY contain a declaration-ordered `i32` tag
and statically typed payload slots for the admitted alternatives. Inactive slots
SHALL never be observed as Topal values, and an invalid tag SHALL fail closed.
Definitions, calls, and returns SHALL use one exact private LLVM type and leave
target-physical aggregate lowering to LLVM. This representation SHALL NOT be a
public or foreign sum ABI, a serialization identity, or a compiled-library
metadata key. DWARF and the bundled GDB renderer SHALL preserve the nominal
type, active alternative, and active payload without presenting inactive
storage as a source value.

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

### TOPAL-COMPILER-EFFECT-EMPTY-001 — Canonical empty Effect value

`Effects ()` SHALL produce the canonical empty value classified by `Effect`
without performing a runtime interaction. The compiler SHALL keep that checked
identity distinct from Unit and `Completed` through bindings, same-type
equality, decomposed positional products, scalar function parameters and
results, canonical display, DWARF, and GDB. The Effect-only scalar subset SHALL
NOT by itself claim aggregate function-result or Effect-list support;
`TOPAL-COMPILER-TUPLE-RESULT-001` separately admits a qualified Tuple result
containing this value.

The empty row MAY use a sealed singleton machine carrier, but that carrier SHALL
NOT become a public integer ABI or make distinct Topal types interchangeable.
Construction and passage SHALL require no allocation or effect-specific runtime
call. Display SHALL use only the Topal-owned platform write boundary. This
increment SHALL add no foreign dependency, other-language standard library, or
native ABI revision.

### TOPAL-COMPILER-FUNCTION-EMPTY-EFFECT-001 — Explicit empty function effect bound

For an otherwise admitted v0.1 ordinary function, the compiler SHALL accept an
explicit post-result `: Effects ()` upper bound, infer the exact empty effect row
for its admitted implementation, verify containment before code generation,
and retain the distinction between an absent bound and an explicitly declared
empty bound in the checked function metadata. A nonempty, alternative,
polymorphic, or otherwise unsupported effect bound SHALL be rejected rather
than erased or assumed empty. Existing `Decreases` proof annotations SHALL
remain governed by their separate compiler rule.

For an exact single visible overload carrying that explicit empty bound, the
compiler SHALL admit `lang view function` as a typed static `lang FunctionView`
which retains the root identity, input classifiers, result classifier,
staticness, and canonical declared empty row. The view MAY be bound or
discarded during compilation, but SHALL be erased before LLVM lowering. A
runtime observation, function/aggregate boundary, root data export, zero- or
multi-overload view, and every other introspection form SHALL remain rejected
until a rule admits its complete static-to-runtime behavior.

The generated function SHALL use the same direct private signature, code, and
runtime debug information as an equivalent inferred-empty function. LLVM IR,
DWARF, and the linked executable SHALL contain no FunctionView object, effect
descriptor, reflection registry, dispatch, hidden effect argument, foreign
runtime, C/C++ standard library, undefined symbol, needed library, relocation,
public/serialized/library effect ABI, or native ABI revision.

### TOPAL-COMPILER-LIST-EFFECT-001 — Immutable Effect List foundation

Within the admitted `List Effect` subset, an immediate classifier context SHALL
determine the element type of `Empty` and of every nested
`Entry (value, remaining)` constructor. The compiler SHALL require every value
to be `Effect`, every remaining value to be `List Effect`, evaluate constructor
fields in source order, and retain the exact List classifier through immutable
bindings and ordinary function parameters and results. Display SHALL produce
the same recursive `Entry`/`Empty` spelling as the interpreter.

On Linux x86-64, `Empty` MAY be a null private pointer and `Entry` MAY be an
immutable, naturally aligned node containing the sealed Effect carrier and the
remaining-node pointer. Nodes SHALL be created through the Topal-owned mapping
boundary and remain valid for their process-lifetime use. Private definitions,
calls, and returns SHALL use one exact pointer prototype and leave physical
AMD64 argument and result placement to LLVM. DWARF and the bundled GDB renderer
SHALL preserve and safely render the source `List Effect` identity.

The node shape SHALL NOT be a public foreign ABI, serialized library identity,
or promise for another element type. This rule SHALL NOT imply List equality,
decisions, traversal, mutation, or a final reclamation policy, and SHALL add no
foreign allocator, C/C++ runtime, other-language standard library, or native
ABI revision.

### TOPAL-COMPILER-LIST-INT-CONTAINMENT-001 — Exact Int List containment

The compiler SHALL extend contextual homogeneous construction, immutable
bindings, canonical display, and ordinary private function parameters and
results to `List Int`. It SHALL preserve every arbitrary-precision Int exactly.
For each admitted containment expression, it SHALL evaluate the complete List
operand and then the entry or List pattern operand exactly once.

`contains-entry` SHALL return true exactly when an equal Int entry occurs.
`contains-sequence` SHALL return true exactly when the pattern occurs as a
consecutive sequence. `contains-subsequence` SHALL return true exactly when the
pattern occurs in order while permitting gaps. Empty sequence and subsequence
patterns SHALL match every List. Equality of entries SHALL use canonical Int
equality, and no operation SHALL mutate an input.

On Linux x86-64, an Int List node MAY use the same private 16-byte size as the
Effect List node, with an exact Int pointer and remaining-node pointer in its
two words. Containment SHALL use allocation-free finite loops and SHALL NOT
depend on LLVM optimization for correctness. Private calls and returns SHALL
use one exact pointer prototype with physical placement selected by LLVM.
DWARF and the bundled GDB renderer SHALL preserve and safely render the
`List Int` identity and arbitrary-precision entries.

This rule SHALL NOT create a public, foreign, serialized, or generic List ABI;
promise a final reclamation policy; or add a foreign allocator, C/C++ runtime,
other-language standard library, or native ABI revision. It SHALL NOT imply
List equality, decisions, other observations or transformations, or another
element classifier.

### TOPAL-COMPILER-LIST-INT-REMOVAL-001 — Immutable Int List removal

For `List Int`, the compiler SHALL evaluate the complete List operand and then
the removal value exactly once. `remove-first` SHALL remove only the earliest
equal entry; if no entry is equal, it SHALL preserve the List unchanged.
`remove-all` SHALL remove every equal entry. Both operations SHALL retain the
relative order and exact arbitrary-precision values of all remaining entries,
preserve the `List Int` classifier, and leave the input List immutable.

The Linux x86-64 lowering SHALL use finite nonrecursive loops and canonical Int
comparison. It MAY share an unchanged input or suffix and MAY build retained
nodes in a contiguous Topal-owned allocation, provided every resulting logical
node retains the private Int-pointer/remaining-pointer shape. Correctness at O0
SHALL NOT depend on LLVM optimization. Specialized layout and removal fragments
SHALL be included only when required by checked expressions, without adding a
foreign allocator, runtime, standard library, undefined symbol, needed library,
or relocation.

This rule SHALL NOT expose or stabilize the private node or allocation layout;
revise the native ABI; promise reclamation beyond process lifetime; or imply
List equality, decisions, another transformation, or another element
classifier. DWARF and GDB SHALL continue to describe and render the complete
semantic `List Int` value.

### TOPAL-COMPILER-LIST-INT-CORE-001 — Basic immutable Int List operations

For the admitted `List Int` specialization, the compiler SHALL implement
explicit empty and singleton construction, prepend, append, concatenation,
reverse, entry count, emptiness, structural equality, total first/rest/uncons
projections, and complete `Empty`/`Entry (first, rest)` decisions according to
their container rules. Every operand SHALL be evaluated exactly once in source
order. Every result SHALL preserve exact arbitrary-precision Int values and
the `List Int` classifier, and no operation SHALL mutate an input List.

The Linux x86-64 lowering SHALL use the existing private Int-pointer/remaining-
pointer node. Concatenation MAY share its right operand but SHALL copy the left
operand before linking it; append and reverse SHALL publish newly constructed
immutable nodes. Observations and decisions SHALL traverse or project finite
nodes without recursion. Equality SHALL use canonical Int comparison and
terminate only after observing a mismatch or both ends. Correctness at O0 SHALL
NOT depend on LLVM optimization.

`first`, `rest`, and `uncons` SHALL use the existing private Optional header;
their admitted payloads SHALL respectively preserve `Int`, `List Int`, and
`(Int, List Int)`, including `Some Empty` without confusing it with `None`.
DWARF and the bundled GDB renderer SHALL expose those semantic Optional types
and render their complete values safely.

Specialized core helpers SHALL be included only when a checked expression
requires them. This rule SHALL add no foreign allocator, C/C++ runtime,
other-language standard library, undefined symbol, needed library, relocation,
or native ABI revision. It SHALL NOT stabilize the private node, Optional, or
pair layout; create a public, foreign, serialized, persistent, or generic List
ABI; promise reclamation beyond process lifetime; admit another element
classifier; or imply remaining List algorithms.

### TOPAL-COMPILER-LIST-INT-FUNCTIONS-001 — Contextual Int List functions

For `List Int`, the compiler SHALL admit contextual binding-pattern anonymous
functions for `map`, `select`, and `fold` when their checked scalar types are
respectively `Int -> Int`, `Int -> Boolean`, and `(Int, Int) -> Int`. The
compiler SHALL evaluate the List once, evaluate the fold initial state once
after the List, and specialize each anonymous body at its collection use. Any
admitted enclosing lexical value referenced by a body SHALL retain its ordinary
immutable captured value.

Map and select SHALL invoke their body exactly once for each source entry in
List order. Map SHALL return every transformed Int in that order. Select SHALL
return exactly the original Int values whose predicate is true, without
changing their relative order. Fold SHALL pass the preceding state and current
entry in that order, return the initial state for Empty, and otherwise return
the final state. Inputs SHALL remain immutable and every arbitrary-precision
Int SHALL remain exact.

On Linux x86-64, lowering SHALL use finite in-module LLVM loops over the private
List node. Map and select MAY initialize and link fresh nodes incrementally
while the result is inaccessible, but SHALL publish only an immutable List.
Fold state SHALL remain a private exact Int pointer. No operation SHALL require
a callback ABI, runtime function value, indirect call, traversal dispatcher,
host stack recursion, or LLVM optimization for correctness. DWARF and GDB SHALL
retain source List/Int identities and inspectable result bindings at O0.

This rule SHALL add no foreign allocator, C/C++ runtime, other-language
standard library, undefined symbol, needed library, relocation, or native ABI
revision. It SHALL NOT create a public, foreign, serialized, persistent, or
generic collection/callable ABI; stabilize private node allocation; promise
reclamation beyond process lifetime; admit another List element or fold-state
classifier; admit anonymous product patterns; or imply early traversal control
or remaining collection algorithms.

### TOPAL-COMPILER-LIST-INT-BOUND-FUNCTIONS-001 — Bound anonymous List functions

The compiler SHALL extend the admitted `List Int` map/select/Int-state fold
operations to an anonymous Function value first bound to an immutable name.
The binding SHALL retain its anonymous parameter pattern, body, defining static
context, and immutable lexical-capture snapshot. Each collection use SHALL
infer and enforce the same exact scalar signature as a directly contextual
body while preserving the bound Function's distinct source value and display
identity.

The Linux x86-64 backend SHALL specialize the retained body into the same
finite source-ordered LLVM loops as the contextual form. Evaluating the
Function binding MAY retain its private debug/display identity tag, but
collection execution SHALL NOT dispatch on that tag or require a function
pointer, callback convention, indirect call, closure allocation, or callable
runtime. Behavior and rejection SHALL remain exact at O0 without optimization.
DWARF and GDB SHALL preserve both bound Function identities and resulting
List/Int values.

This rule SHALL NOT create a public, foreign, serialized, persistent, or
generic callable/collection ABI; stabilize a Function tag or List layout;
admit named, symbolic, product-pattern, escaping, or dynamically selected
collection functions; admit other element/result/state classifiers; or add a
foreign allocator, C/C++ runtime, other-language standard library, undefined
symbol, needed library, relocation, or native ABI revision.

### TOPAL-COMPILER-RANGE-SELECTION-001 — Private range selection

The compiler SHALL admit `List Int` value selection and zero-based index
selection by `Range Int`. It SHALL evaluate the List and Range once in source
order, traverse the finite List once in source order, apply the exact retained
endpoint inclusivity rules, preserve multiplicity, and return a fresh immutable
List containing exactly the selected entries. Value selection SHALL compare the
arbitrary-precision entry; index selection SHALL compare the exact nonnegative
position. Empty, disjoint, and inverted ranges SHALL produce Empty without
changing the source List.

The compiler SHALL also admit String index selection when both the String and
the finite `Range Int` value are closed and compiler-known. It SHALL select by
the same pinned user-perceived Character segmentation as the language and emit
the result as an ordinary String. This static semantic evaluation SHALL remain
correct at O0 and SHALL NOT expose `SelectionOf`, `RangeSelectionOf`, `SliceOf`,
or a storage representation. Dynamic String range selection SHALL remain an
explicitly rejected later increment until a freestanding Topal Unicode
segmentation runtime is available; LLVM provides no equivalent semantic
facility.

The Linux x86-64 lowering SHALL keep List selection in a conditional private
runtime fragment backed only by the Topal allocator, exact Int/Range helpers,
and Linux syscall platform boundary. It SHALL add no foreign allocator, C/C++
runtime, other-language standard library, undefined symbol, needed library,
relocation, public/serialized/persistent/generic collection ABI, stabilized
private layout, or native ABI revision. DWARF and GDB SHALL preserve the source
List, Range, String, selected values, and user frames.

### TOPAL-COMPILER-TRAVERSAL-CONTROL-001 — Private traversal control

For an admitted `List Int` fold with an `Int` initial state, the compiler SHALL
admit an action result of either `Int` or `TraversalControl Int`. `Continue
state` SHALL carry its exact Int payload into the next reached action. `Finish
result` SHALL return its exact Int payload from the fold immediately, without
evaluating an action for any later entry. Empty SHALL return the once-evaluated
initial state. Construction SHALL evaluate each payload exactly once, and the
ordinary Int-result fold behavior SHALL remain unchanged.

On Linux x86-64, `Continue Int` and `Finish Int` SHALL use a private immutable
16-byte object with an unsigned tag at offset zero and the canonical Int pointer
at offset eight. The specialized fold SHALL inspect that object in generated
LLVM control flow and branch directly to either its state-advance or result
path. Correctness at O0 SHALL NOT depend on optimization, a traversal runtime
dispatcher, callback convention, indirect call, or host recursion. DWARF and
the bundled GDB renderer SHALL expose the semantic `TraversalControl Int`
identity and safely distinguish both constructors.

This rule SHALL add no foreign allocator, C/C++ runtime, other-language
standard library, undefined symbol, needed library, relocation, public,
foreign, serialized, persistent, or generic traversal-control ABI, stabilized
private layout, or native ABI revision. Other payload and collection
classifiers, traversal-control function boundaries, generators, and remaining
traversal algorithms SHALL remain rejected pending later increments.

### TOPAL-COMPILER-LIST-INT-PAIR-MAP-001 — Int-pair List product map

The compiler SHALL admit contextual construction of `List (Int, Int)` and a
`map` action whose single two-field anonymous product pattern binds both exact
Int fields. The action MAY be directly contextual or retained in an immutable
anonymous Function binding. It SHALL run once for every pair in source order,
bind the first and second fields in pattern order, and produce an Int for the
corresponding output entry. Duplicate bindings and a product-pattern arity
other than two SHALL be diagnosed. Inputs SHALL remain immutable, and every
arbitrary-precision Int SHALL remain exact.

On Linux x86-64, a private pair-List node SHALL contain the two canonical Int
pointers inline at offsets zero and eight followed by its remaining-node
pointer at offset sixteen. Construction SHALL allocate and initialize the
complete 24-byte node only after evaluating its value and remaining List in
source order. The specialized generated map loop SHALL load and bind both
fields directly and SHALL retain the existing immutable `List Int` result
construction. Canonical output, DWARF, and the bundled GDB renderer SHALL
preserve `List (Int, Int)` and its source-shaped pair entries at O0.

This rule SHALL add no tuple payload allocation, generic or type-erased List
runtime, callback ABI, indirect call, foreign allocator, C/C++ runtime,
other-language standard library, undefined symbol, needed library, relocation,
public/foreign/serialized/persistent/generic List ABI, stabilized private
layout, or native ABI revision. Pair-List function boundaries, other product
shapes and classifiers, pair-returning transformations, and remaining
operations over pair Lists SHALL remain rejected pending later increments.

### TOPAL-COMPILER-LIST-RECURSIVE-001 — Exact recursive Int/String Lists

The compiler SHALL admit contextual immutable construction of `List (Int,
String)` and `List List (Int, String)`. It SHALL admit the outer recursive List
as an ordinary function parameter and result, and SHALL preserve the complete
classifier and value through a direct call. For that outer List, `first` SHALL
produce `Optional (List (Int, String))`, `entry-count` SHALL produce the exact
finite Int count, and equality SHALL recursively compare outer length and
order, inner length and order, every arbitrary-precision Int, and every exact
String. `Some Empty` SHALL remain distinct from `None`. Canonical output SHALL
match the interpreter's nested `Entry`/`Empty` spelling. Every source operand
and constructor field SHALL be evaluated exactly once in source order.

On Linux x86-64, an inner pair node SHALL contain the canonical Int pointer at
offset zero, the immutable String pointer at offset eight, and its remaining-
node pointer at offset sixteen. An outer node SHALL contain the inner-List
pointer at offset zero and its remaining-node pointer at offset eight. Empty at
either level SHALL remain null. Construction SHALL publish only completely
initialized private nodes allocated through the Topal Linux mapping boundary.
The compiler SHALL select exact nonrecursive LLVM loops for outer projection,
counting, and structural equality; inner equality SHALL use the canonical Int
and String comparators. Correctness at O0 SHALL NOT depend on optimization,
host recursion, an indirect call, callback, runtime type tag, or generic or
type-erased List operation.

The outer function definition and call SHALL use one matching module-private
LLVM pointer prototype and leave physical AMD64 argument/result placement to
LLVM. Target-layout-derived DWARF and the bounded validating GDB renderer SHALL
expose both recursive semantic List identities and complete nested values.
This rule SHALL add no public, foreign, serialized, persistent, or compiled-
library List layout; stabilized private layout; C/C++ runtime; other-language
standard library; foreign allocator; undefined symbol; needed library;
relocation; or native ABI revision. Direct inner pair-List function passage
and equality, deeper recursion, outer `rest`/`uncons`, other product shapes and
element classifiers, remaining List algorithms, reclamation beyond process
lifetime, and versioned library metadata/adapters remain deferred.

### TOPAL-COMPILER-TUPLE-RESULT-001 — Private positional-product results

An ordinary or static function result classified by a recursively composed
Tuple SHALL preserve every field in source order when every leaf type has an
admitted exact private representation. The function body SHALL evaluate
once according to the existing block and return rules, and its caller SHALL
receive the same complete Tuple without field erasure, integer substitution,
semantic heap storage, runtime allocation, or dependence on optimization.

For Linux x86-64, the backend SHALL express this result as a non-packed LLVM
struct whose fields recursively use those private value types. The definition
and every call SHALL have one exactly matching private calling convention and
prototype; LLVM SHALL select the physical register or stack transport under the
qualified target and data layout. This representation SHALL NOT be exposed as
a stable compiled-library or foreign ABI. Record fields admitted by
`TOPAL-COMPILER-RECORD-BOUNDARY-001` MAY occur recursively; any Tuple containing
another unsupported leaf SHALL remain rejected.

DWARF SHALL describe the source Tuple, its ordered fields, and its target-exact
layout. Named Tuple bindings SHALL remain inspectable in GDB at O0; a
debug-only stack shadow MAY be used when LLVM cannot preserve a direct SSA
aggregate location. Such a shadow SHALL NOT become the semantic representation
or require a runtime, foreign allocator, C/C++ library, other-language standard
library, or native ABI revision.

### TOPAL-COMPILER-TUPLE-DECISION-001 — Field-wise Tuple control-flow joins

Every otherwise-admitted complete decision family MAY produce a common
recursively composed Tuple type supported by
`TOPAL-COMPILER-TUPLE-RESULT-001`. The checked compiler model SHALL require the
complete structural type of every action to be identical after its existing
canonical conversions. Subject evaluation, rule order, matcher evaluation, and
selection of exactly one delayed action SHALL remain unchanged at O0.

The backend SHALL join the selected decomposed value independently at each
machine-represented leaf with a correctly typed LLVM `phi`; Unit leaves require
no machine join. It SHALL preserve Tuple nesting and field order without an
aggregate `phi`, eager action evaluation, semantic aggregate storage, heap
allocation, runtime helper, foreign dependency, or ABI revision. Record values
admitted by `TOPAL-COMPILER-RECORD-BOUNDARY-001` MAY occur recursively; Tuples
with other unsupported leaves SHALL remain rejected.

### TOPAL-COMPILER-TUPLE-PARAMETER-001 — Private positional-product parameters

An ordinary or static function parameter classified by a recursively composed
Tuple SHALL preserve every field in source order when every leaf type has an
admitted exact private representation. The argument expression SHALL be
evaluated once before selection and entry. A one-parameter candidate SHALL
match the complete positional product, while a multi-parameter candidate SHALL
match its fields in declaration order; source-ordered overload selection SHALL
remain unchanged when both candidate shapes apply.

For Linux x86-64, the backend SHALL pass each admitted Tuple parameter as one
non-packed LLVM struct whose fields recursively use those private value types.
The caller SHALL form the aggregate with `insertvalue`, the callee SHALL recover
its decomposed source value with `extractvalue`, and the definition and every
call SHALL have one exactly matching private `fastcc` prototype. LLVM SHALL
select physical register or stack transport for the qualified target and data
layout. This representation SHALL NOT be a stable compiled-library or foreign
ABI and SHALL require no semantic aggregate storage, heap allocation, runtime
helper, foreign dependency, other-language standard library, or native ABI
revision.

DWARF SHALL describe the complete source Tuple parameter with target-exact
field layout. A named parameter SHALL remain inspectable in GDB at O0; a
target-aligned debug-only stack shadow and `#dbg_declare` MAY be used when the
aggregate SSA argument is not directly inspectable. A discarded Tuple
parameter SHALL retain its checked and LLVM signature position but SHALL NOT be
unpacked or receive a source or DWARF binding. Record fields admitted by
`TOPAL-COMPILER-RECORD-BOUNDARY-001` MAY occur recursively; Tuples with other
unsupported leaves SHALL remain rejected.

### TOPAL-COMPILER-TYPE-VALUE-001 — Closed fundamental Type values

The compiler SHALL admit the fundamental `Boolean`, `Int`, `Nat`, `Rational`,
`String`, `Unit`, and `Scope` names in expression position as distinct immutable
values classified by `Type`. It SHALL preserve their canonical identities
through bindings, same-kind equality, decomposed products, scalar function
parameters and results, canonical display, DWARF, and GDB. This closed subset
SHALL NOT imply runtime reflection or admission of user-defined Type values.

A target lowering MAY use private tags for the closed set, but tag numbers SHALL
NOT be exposed as a public foreign ABI or serialized library-metadata identity.
Future library metadata SHALL identify types canonically and independently of
the private machine representation. This increment SHALL require no registry,
allocation, foreign type-information runtime, other-language standard library,
or native ABI revision.

### TOPAL-COMPILER-LAYOUT-POLICY-001 — Closed external-layout policy values

The compiler SHALL resolve the closed external-layout policy spellings as
seven distinct nominal enum types with these declaration orders:

- `Endian`: `Little`, `Big`;
- `Access`: `ReadWrite`, `ReadOnly`, `WriteOnly`, `Reserved`;
- `BitOrder`: `MostSignificantFirst`, `LeastSignificantFirst`;
- `Packing`: `Natural`, `Packed`;
- `FieldOrder`: `Declared`;
- `PayloadPlacement`: `AfterTag`, `Overlay`; and
- `LayoutPolicy`: `NoLength`, `NoTerminator`.

Bindings, decomposed products, same-type equality, canonical display, DWARF,
and GDB SHALL preserve those exact nominal identities and source labels.
`NoLength` and `NoTerminator` SHALL remain distinct values of the same nominal
type. Values from different families SHALL remain nominally distinct, and
cross-family equality SHALL be rejected.

On Linux x86-64, the backend MAY reuse the private declaration-ordered `i32`
tag lowering for closed nominal enums. Those tags SHALL be compiler-selected
machine representation only: they SHALL NOT construct or inspect an external
layout, encode or serialize data, designate a location or address, grant access
authority, or become a public, foreign, persistent, or compiled-library ABI.
Future library metadata SHALL identify every policy and value canonically and
independently of its private tag. This increment SHALL require no layout
runtime, allocator, foreign dependency, C/C++ runtime, other-language standard
library, needed library, dynamic relocation, or native ABI revision.

### TOPAL-COMPILER-STATIC-INTROSPECTION-001 — Closed static introspection foundation

For a v0.1 source context without optional features, the compiler SHALL admit
`lang identity` and `lang view` over each closed fundamental Type admitted by
`TOPAL-COMPILER-TYPE-VALUE-001` when named directly. The checked identity SHALL
retain the `Type` object kind and canonical `type:<name>` identity; the checked
view SHALL retain
the typed primitive Type-view form and semantic Type identity. `lang context`
SHALL retain the exact `topal` language identity, active numeric v0.1 Version,
and empty feature set. These results SHALL be static-only typed compiler values:
binding or discarding them SHALL produce no LLVM value, DWARF variable or type,
descriptor, or executable metadata. Runtime use or containment SHALL be
rejected.

For two directly named admitted fundamental Type operands, `lang same-object` and
`lang equivalent-type` SHALL evaluate during checking from canonical semantic
identity and produce an ordinary Boolean constant. A runtime operand or an
unadmitted object kind SHALL be rejected rather than assigned a missing or
textual identity. `lang version` SHALL produce the context's ordinary numeric
`Version`, containing the four nonnegative `major`, `minor`, `patch`, and
`build` components and using the canonical abbreviated display.

On Linux x86-64, this initial Version value MAY use a compiler-private pointer
to four immutable Nat carriers materialized in the current frame. DWARF SHALL
describe the same four-field Version and the bundled GDB renderer SHALL safely
produce its canonical source spelling. The executable SHALL remain free of
load-time relocations, undefined symbols, needed libraries, foreign runtimes,
C/C++ standard libraries, and reflection or Version runtime helpers. The
private layout SHALL NOT become a function, public, foreign, serialized, or
compiled-library ABI, and SHALL NOT revise the native ABI. General Type views,
other introspection subjects and relations, later context changes, deliberate
static-to-runtime conversion, and Version operations or function boundaries
remain unsupported.

### TOPAL-COMPILER-CAPABILITY-COMPOSE-001 — Closed static Capability composition

For a v0.1 source context without optional features, the compiler SHALL admit
the atomic `Equality`, `Ordering`, `Foldable`, `Membership`, `Indexed`, and
`Keyed` Capability values and root bindings formed from them with `and` and
`or`. An explicit `Capability` classifier SHALL be checked without changing the
value. `and` SHALL form the cross-product of alternatives and retain every
atomic promise in each resulting conjunction. `or` SHALL retain the alternatives
from both operands. Within this closed subset, conjunction members and
alternatives SHALL be canonical, order-independent, and idempotent, in
conformance with `TOPAL-CAPABILITY-COMPOSE-001`.

Each Capability SHALL remain static checked metadata and SHALL create no
runtime method namespace, evidence object, table, tag, descriptor, allocation,
symbol, or DWARF variable or type. Root bindings and discarded values SHALL
lower to no LLVM instruction. When the complete source-entry result is exactly
one admitted Capability, the compiler MAY produce its canonical textual
observation as a constant output literal; this SHALL NOT materialize the
Capability as a runtime value. Capability containment in another value,
non-root binding, function or other machine boundary, unsupported atom,
application, evidence claim, or operator SHALL remain rejected.

The generated Linux x86-64 executable SHALL remain free of undefined symbols,
needed libraries, and load-time relocations and SHALL use only the existing
Topal-owned syscall writer for the final observation. It SHALL introduce no
foreign runtime, C/C++ standard library, public or foreign Capability ABI,
serialized representation, compiled-library metadata format, or native-ABI
revision. Future library metadata SHALL encode canonical semantic Capability
identities in a separately versioned schema rather than reuse a target lowering.

### TOPAL-COMPILER-FUNCTION-INTERFACE-001 — Closed direct function-interface conformance

For a source-root v0.1 `Interface` whose uniquely named operations contain only
ordinary function shapes over admitted private native parameter and result
classifiers, the checked compiler model SHALL retain the nominal `root.Name`
interface identity and a canonical operation set containing every operation
name, parameter classifier sequence, and result classifier. A following direct
interface construction SHALL contain exactly one ordinary function declaration
for every operation, no additional declaration, and no duplicate role. Each
shape SHALL match exactly, and successful conformance SHALL retain a mapping
from every interface role to its exact root declaration identity. Any admitted
explicit empty effect bound SHALL remain attached to that declaration evidence;
an absent bound SHALL NOT be mislabeled as inferred evidence for an unchecked
body.

The selected functions SHALL use the same private direct LLVM lowering as
ordinary root functions. The interface and conformance evidence SHALL be
erased before LLVM: they SHALL produce no runtime interface value, namespace,
vtable, function pointer, indirect call, dispatcher, tag, descriptor,
allocation, symbol, relocation, DWARF type, or DWARF variable. DWARF and GDB
SHALL retain the actual implementation function, parameters, source locations,
and frames. This behavior SHALL be required at `-O0` and SHALL NOT depend on an
LLVM optimization.

The executable SHALL remain freestanding and SHALL introduce no undefined
symbol, needed library, foreign runtime, C/C++ standard library, or native-ABI
revision. Source visibility SHALL NOT by itself create a native-artifact export.
Until the separately versioned compiled-library interface/evidence schema is
implemented, these checked records SHALL NOT populate or reinterpret the
native artifact's interface digest, export, or evidence fields. Packaged or
dynamically selected implementations, generator operations, non-root contexts,
message implementations, v0.2 contracts, and cross-library consumption SHALL
remain rejected rather than receive a provisional runtime or serialized ABI.

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
foreign aggregate ABI. `TOPAL-COMPILER-RECORD-BOUNDARY-001` separately admits a
private aggregate carrier at function and control-flow boundaries and a
truthful debug-only stack shadow; persistent semantic Record storage remains
unsupported.

### TOPAL-COMPILER-RECORD-BOUNDARY-001 — Order-preserving private Record boundaries

An ordinary or static function parameter or result classified by a closed
structural Record SHALL preserve every field value, exact classifier, and the
value's construction order when every recursively nested Tuple or Record leaf
has an admitted private representation. Record classifier identity and
matching SHALL use the canonical label set independently of declaration or
construction order. A candidate argument SHALL be evaluated once and ordinary
source-ordered overload selection SHALL remain unchanged.

For Linux x86-64, the backend SHALL represent an admitted Record boundary as a
non-packed LLVM struct containing values in canonical label order followed by
one `i32` canonical-field index for each display position. The indexes SHALL
form a permutation of the complete canonical field set. Caller construction,
callee decomposition, results, and calls SHALL use one exactly matching private
`fastcc` prototype, with LLVM selecting the physical register or stack
transport under the qualified target and data layout. This carrier SHALL NOT
be exposed as a stable compiled-library, public Topal, or foreign ABI.

Every otherwise-admitted decision family MAY produce such a Record. The backend
SHALL join each canonical field recursively and each display-order index with a
correctly typed LLVM `phi`, then use the selected permutation for canonical
display. It SHALL NOT use an aggregate `phi`, eagerly evaluate an unselected
action, allocate semantic storage, or require a Record runtime helper.

DWARF SHALL expose the Record's semantic named fields at their target-exact
offsets and SHALL account for the private order suffix in the complete type
size without presenting those implementation fields as source members. Named
Record bindings and parameters SHALL remain inspectable in GDB at O0; a
target-aligned debug-only stack shadow MAY be used when the SSA aggregate is not
directly inspectable. The carrier and shadow SHALL require no heap allocation,
foreign dependency, C/C++ runtime, other-language standard library, or native
ABI revision.

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

### TOPAL-COMPILER-STRING-CHARACTERS-FOREACH-001 — Closed Character traversal

A root `foreach` whose source is the direct `characters text` application SHALL
admit a plain String whose complete preserved sequence is known during checking
and a capture-free action that binds Character and produces Unit. It SHALL
evaluate the source String exactly once, segment it with the selected language
context's pinned extended-grapheme rules, invoke the action exactly once for
each complete Character in preserved order, resume with Unit after each action,
and return Unit after the
last action. Empty input SHALL invoke no action and return Unit. The immutable
source String SHALL remain unchanged. The statement MAY bind its Unit result,
with an optional exact Unit classifier, or discard that result.

This sequence SHALL be mandatory frontend semantics at `-O0`, not an LLVM
optimization. On Linux x86-64, the backend SHALL lower each statically known
Character through the existing immutable String descriptor and inline the
checked action in source order. DWARF SHALL expose the action binding as
Character, using a debug-only pointer shadow when the action otherwise emits no
machine use. The lowering SHALL create no semantic traversal collection,
Generator object or token, generic Generator runtime, callback, indirect call,
host Unicode dependency, locale dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library Generator
ABI, or `topal-native/6` revision. Dynamic Strings, function-returned Character
generators outside `TOPAL-COMPILER-STRING-CHARACTERS-RESULT-001`, other
non-specializable Character generators, and captured actions remain outside
this specialization. Future compiled-library metadata SHALL identify the
canonical Generator classifier, pinned segmentation identity, action evidence,
linear ownership, and target adapter rather than publish this executable-local
expansion.

### TOPAL-COMPILER-STRING-CHARACTERS-GENERATOR-001 — Named closed traversal

The compiler SHALL admit `characters text` as an exact
`Generator Character Unit Unit` when the complete plain String is known during
checking. A root binding MAY state that exact classifier and SHALL evaluate the
source String exactly once while constructing a fresh linear value. Checking
MAY retain the pinned ordered Character sequence as compilation-session
provenance, but construction SHALL invoke no foreach action and generated code
SHALL NOT mistake its observation value for executable continuation state.

Root foreach SHALL transfer one such locally bound Generator into the closed
Character traversal of `TOPAL-COMPILER-STRING-CHARACTERS-FOREACH-001`, preserve
the same action order and Unit result, and mark the source binding consumed.
Any later source use SHALL diagnose the consumed Generator rather than restart
or copy it. An unconsumed root binding SHALL remain rejected; the narrow
transferred-parameter close is specified separately by
`TOPAL-COMPILER-STRING-CHARACTERS-CLOSE-001`. Generated DWARF and GDB SHALL
expose the binding as `Generator Character Unit Unit` through a compiler-private
observation token.

This executable-local token and retained provenance SHALL introduce no
Generator object, continuation-state allocation, generic Generator runtime,
callback, indirect call, host Unicode or locale dependency, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library Generator ABI, or `topal-native/6` revision. General function
result transfer outside `TOPAL-COMPILER-STRING-CHARACTERS-RESULT-001`, general
parameter use, persistent or serialized identity, and external library
boundaries remain rejected until canonical Generator, segmentation,
operation/evidence, ownership, close, and target-adapter metadata are defined.

### TOPAL-COMPILER-STRING-CHARACTERS-COLLECT-001 — Closed String reconstruction

The compiler SHALL admit direct `characters text collect String` when the
complete plain String is known during checking. It SHALL evaluate `text`
exactly once, retain the selected language context's pinned ordered Character
segmentation as checking evidence, consume the fresh traversal, and return a
plain String with exactly the source's preserved scalar sequence. Empty input
SHALL return `empty String`.

Because the unchanged finite traversal reconstructs the immutable source
exactly, the Linux x86-64 backend MAY forward the existing source String
descriptor as the result. This SHALL be mandatory O0 semantic lowering, not an
LLVM optimization or an assumption that applies to transformed traversals.
DWARF and GDB SHALL expose the result as String.

The lowering SHALL allocate no intermediate List or Generator object and SHALL
invoke no concatenation loop, generic traversal runtime, callback, indirect
call, host Unicode or locale dependency, C/C++ runtime, other-language standard
library, needed library, dynamic relocation, public/library Generator ABI, or
`topal-native/6` revision. Dynamic Strings, transformed traversals, stored
Generator collection, and external library boundaries remain rejected pending
generated Topal Unicode support and canonical traversal, operation/evidence,
ownership, and target-adapter metadata.

### TOPAL-COMPILER-STRING-CHARACTERS-CLOSE-001 — Owned parameter close

The compiler SHALL admit an ordinary called function with exactly one named
`Generator Character Unit Unit` parameter, a Unit result, and a statement-free
Unit body that leaves the parameter untraversed. Passing one locally bound
closed Character generator SHALL transfer and consume the caller binding. On
function exit the callee SHALL deliver the intrinsic close to that owned
built-in continuation and return Unit without yielding another Character.

The checked model SHALL retain an explicit close operation. Because this
specialization has allocated no continuation object or live traversal state,
the Linux x86-64 backend SHALL lower close to no runtime action after accepting
the compiler-private observation token. The private function SHALL use LLVM's
internal `fastcc` lowering for that `i32` token rather than hard-code System V
register placement. DWARF and GDB SHALL expose the semantic Generator parameter,
using debug-only storage when the no-op close otherwise leaves no inspectable
machine location.

This specialization SHALL add no generic Generator runtime, close dispatcher,
allocation, callback, indirect call, unwind dependency, C/C++ runtime,
other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or `topal-native/6`
revision. Returning the parameter, traversal outside the exact specialization
of `TOPAL-COMPILER-STRING-CHARACTERS-PARAMETER-001`, multiple or additional
parameters, static functions, general close handling, and external boundaries
remain rejected pending canonical Generator classifier, state, ownership,
close-domain/provenance, and target-adapter metadata.

### TOPAL-COMPILER-STRING-CHARACTERS-PARAMETER-001 — Specialized parameter traversal

The compiler SHALL admit an ordinary called function with exactly one named
`Generator Character Unit Unit` parameter and a Unit result when its executable
body is exactly one `foreach` over that parameter with a capture-free
Character-to-Unit action. The top-level call argument SHALL be a locally bound
closed Character generator with retained provenance. Calling the function
SHALL transfer and consume the caller binding exactly once.

Each call SHALL create a private specialization whose checked model receives
only that argument's pinned ordered Character sequence. The caller SHALL
evaluate the source String once before the call. The callee SHALL expand the
action once per Character in preserved order, resume with Unit after each
action, exhaust the transferred continuation, and return Unit without
delivering close. Separate calls with different source Strings SHALL NOT share
their retained sequence.

On Linux x86-64, the private function SHALL accept the existing `i32`
observation/ownership token through LLVM `fastcc`; the token SHALL NOT become
cursor or continuation state. DWARF/GDB SHALL expose the Generator parameter
and Character action binding. The backend SHALL hard-code no System V register
placement.

This specialization SHALL add no Generator object/runtime, state allocation,
dispatcher, callback, indirect call, host Unicode or locale dependency, C/C++
runtime, other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or `topal-native/6`
revision. Dynamic or transformed provenance, general function results outside
`TOPAL-COMPILER-STRING-CHARACTERS-RESULT-001`, multiple or additional
parameters, nested calls, other bodies, and external boundaries remain rejected
pending canonical Generator, segmentation, action-evidence, ownership, and
target-adapter metadata.

### TOPAL-COMPILER-STRING-CHARACTERS-RESULT-001 — Specialized function result

The compiler SHALL admit an ordinary nonrecursive called function with exactly
one named String parameter and result classifier
`Generator Character Unit Unit` when its body has no statements and returns
exactly `characters parameter`. Its top-level call argument SHALL have a closed
exact String value. Function exit SHALL transfer the fresh continuation to the
caller without close delivery; the caller SHALL bind and consume the returned
Generator exactly once through an admitted Character traversal.

Each call SHALL create a distinct private specialization and retain only that
argument's pinned ordered Character sequence beside the private symbol for the
duration of checking. The caller SHALL evaluate its String argument once. The
callee SHALL observe the existing immutable String descriptor and return the
existing private Generator token; later caller traversal SHALL use only that
call's retained sequence. Distinct calls SHALL NOT share provenance.

On Linux x86-64, LLVM `fastcc` SHALL select placement for the private String
descriptor argument and `i32` Generator result. The backend SHALL hard-code no
System V register placement. DWARF/GDB SHALL expose the String parameter, the
Generator result classifier/value, the caller traversal Character binding, and
both call frames.

This specialization SHALL add no Generator object/runtime, state allocation,
dispatcher, callback, indirect call, host Unicode or locale dependency, C/C++
runtime, other-language standard library, needed library, dynamic relocation,
public/library calling convention or Generator ABI, or `topal-native/6`
revision. Dynamic/transformed arguments, anonymous/static/recursive/nested or
multi-parameter functions, other result bodies, unbound results, general close
handling, and external boundaries remain rejected pending canonical Generator,
segmentation, construction evidence, ownership, close, and target-adapter
metadata.

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

### TOPAL-COMPILER-CONSTRAINT-VALUE-001 — Named constraint observation values

For an admitted root declaration `Name is Base constraint { parameter }
predicate`, the checked program model SHALL retain `Name`, the supported
primitive `Base`, the predicate parameter, and the checked Boolean predicate.
The predicate SHALL be checked as a pure closed function of that parameter in
this increment. A separately named root binding classified as `Constraint`
SHALL receive that binding's nominal identity while retaining the original base
and predicate, consistently with the shared interpreter.

Generated code MAY represent each retained identity with a deterministic
module-private `i32` tag. Canonical display and DWARF/GDB SHALL use
`<Constraint Name>`. The tag SHALL NOT dispatch or stand in for the retained
predicate, and it SHALL NOT be a public ABI or compiled-library metadata key.
Constraint application/evidence, captured predicates, function or
persistent/public aggregate machine boundaries, and public identities SHALL
remain rejected until their semantic metadata and environment representation
are implemented. This increment SHALL introduce no constraint runtime,
allocation, foreign dependency, C/C++ runtime, other-language standard library,
or native ABI revision.

### TOPAL-COMPILER-CONSTRAINT-VALIDATE-001 — Int constraint validation

Applying an admitted named Int constraint to a closed exact operand SHALL
evaluate its retained checked predicate during frontend analysis. Acceptance
SHALL retain a distinct refined classifier over the unchanged Int machine
value; rejection SHALL diagnose `E-CONSTRAINT-REJECTED`. Equality, ordering,
and arithmetic over an admitted refined value SHALL explicitly forget the
evidence and use exactly the canonical Int operations.

Applying the same constraint to an unknown Int SHALL evaluate the predicate
exactly once in generated code. It SHALL return the existing
`Result (Int, lang arithmetic ArithmeticErrorCode)` representation: success
contains the unchanged operand and failure contains `out-of-range` in domain
`root.Name(Int)` with source provenance. This behavior SHALL remain mandatory
at O0 and SHALL not depend on an LLVM optimization.

The refined source classifier SHALL be present in DWARF while using the exact
base pointer representation. Constraint application SHALL introduce no second
numeric value, predicate dispatcher, constraint runtime, foreign dependency,
C/C++ runtime, other-language standard library, public evidence ABI, or native
ABI revision. Other bases, captured or dependent predicates, evidence across
function or persistent/public aggregate machine boundaries, and dynamically
selected constraint identities remain outside this increment.

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
