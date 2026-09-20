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

### TOPAL-COMPILER-LINT-VARIANT-001 — Authority-free lint context

For a v0.1 source context whose only canonical optional feature is `lint`, the
compiler SHALL retain the exact feature set in its checked program and in an
admitted static `lang context` value. `lang lint` SHALL be admitted exactly when
that feature is selected and SHALL produce an empty `Scope` named `lang lint`.
Without the feature, the compiler SHALL diagnose `E-LINT-VARIANT`; it SHALL
reject every other optional feature until a compiler rule admits that variant.

On Linux x86-64, the backend MAY represent the lint namespace as a
compiler-private `i32` alternative of the existing `Scope` enumeration.
Canonical output and DWARF/GDB SHALL expose `<namespace lang lint>`. The tag
SHALL NOT grant or encode filesystem, network, process, debugger, application,
or lint-execution authority, and lowering SHALL introduce no namespace table,
dynamic lookup, allocation, foreign dependency, C/C++ runtime, other-language
standard library, public ABI, or native ABI revision.

A future compiled-library interface containing this context SHALL identify its
language revision, canonical variant feature set, exposed vocabulary, and
authority profile independently of private tags and symbol spelling. This rule
does not define that interface or admit dynamic lint APIs.

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
interpreter. Unbounded ranges and range-based collection selection SHALL remain
outside this rule until their prerequisite semantics and value representations
are implemented. Exact Int and Rational infinity endpoints are admitted only
under `TOPAL-COMPILER-INFINITY-001`.

### TOPAL-COMPILER-INFINITY-001 — Contextual exact infinities

The compiler SHALL admit exact `+Infinity` and `-Infinity` in an immediate
root-binding `Int` context and `+Infinity` in an immediate root-binding `Nat`
context. It SHALL also admit either infinity in an immediate root-binding
`Rational` context. Checked code SHALL retain direction and source classifier,
and SHALL implement canonical display, equality, ordered predicates, three-way
comparison, and explicitly bounded `Range Int` and `Range Rational`
construction, membership, intersection, emptiness, and bound observation with
either same-domain infinity as an endpoint. A finite Int beside a Rational
infinity SHALL use the canonical finite embedding. The semantics SHALL hold at
`-O0` without relying on an optional LLVM optimization. Closed root-scope
negation, absolute value, addition, subtraction, and multiplication SHALL admit
every statically proved total case in `TOPAL-NUM-INFINITY-ARITHMETIC-001` and
SHALL diagnose every statically evident indeterminate case. When an Int or
Rational multiplication has exactly one admitted infinity operand and the
finite factor's zero-ness is not statically provable, it SHALL have the
ordinary arithmetic `Result` classifier: zero SHALL return code
`indeterminate` with source provenance and nonzero SHALL return the correctly
signed infinity. Specialized executable-private non-recursive functions SHALL
admit exact Int, Nat, and Rational infinity parameters/results and captured
environments, including recursively corresponding Tuple and Record fields,
while preserving domain and direction evidence for checked arithmetic and
debugging. A context-free constant, negative Nat infinity, other dynamic
indeterminate Result path,
other infinity arithmetic, implicit cross-domain Int/Rational infinity
conversion, recursive infinity entry, or persistent, serialized, public, or
compiled-library infinity boundary SHALL be rejected explicitly in this
increment.

Linux x86-64 lowering MAY use immutable executable-private sentinel Int
objects with reserved non-finite tags. Those objects and validated Rational
wrappers MAY cross LLVM-internal specialized function signatures but SHALL NOT
cross public or foreign signatures. Existing finite Int objects and their
`topal-native/6` function representation SHALL remain unchanged. Exact
comparison and output SHALL recognize the sentinels before finite limb logic;
negation, absolute value, addition, subtraction, and multiplication SHALL
recognize them before finite zero, sign, length, or limb logic. Dynamic
multiplication helpers SHALL validate the infinity operand before constructing
Result success or failure. Range operations SHALL reuse the existing opaque
exact endpoint pointers. A violated checked indeterminate invariant SHALL fail
closed rather than treating a sentinel as finite storage.
Rational infinity MAY use a runtime-constructed Rational wrapper whose
numerator is the matching sentinel and whose denominator is canonical one;
construction, arithmetic normalization, comparison, display, and debugging
SHALL recognize that invariant before finite greatest-common-divisor or
cross-multiplication logic. No pointer-bearing infinity object may require a
loader-applied absolute relocation. The runtime SHALL use only Topal-owned
storage and direct qualified Linux syscalls, with no undefined helper, foreign
runtime, C/C++ runtime, other-language standard library, dynamic dependency, or
public ABI.

DWARF SHALL preserve the source `Int`, `Nat`, `Rational`, `Range Int`, and
`Range Rational` identities. The bundled GDB renderer SHALL validate the
reserved tags, zero-length sentinel invariant, and canonical Rational wrapper
before rendering either infinity. Future library metadata carrying infinity
SHALL describe the language revision, semantic numeric domain, direction,
applicable operations and errors, type constraints, ownership, debug contract,
and target adapter independently of LLVM types, private tags, headers, and
symbol names.

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
close handling; nested construction; ordinary-function construction or
Generator parameter/result transfer outside
`TOPAL-COMPILER-GENERATOR-NESTED-FUNCTION-BOUNDARY-001`; repeated consumption;
abandonment; libraries; and external
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
statements; close handling; nested construction; ordinary-function construction
or Generator parameter/result transfer outside
`TOPAL-COMPILER-GENERATOR-NESTED-FUNCTION-BOUNDARY-001`; repeated consumption; abandonment;
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

### TOPAL-COMPILER-GENERATOR-RECURSIVE-NOMINAL-001 — Recursive nominal value directions

The compiler SHALL admit the declaration `Choice is Enum (First, Second)` and
a root custom generator with one named
`(Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))`
initial parameter and the same yield and result directions with Unit
resumption. Its body SHALL consist only of one discarded `yield initial`
followed by `(Some Second, Second)`. The shared regression SHALL start the
generator with exact `(Some First, First)`; the fresh result SHALL be bound and
consumed exactly once by a foreach action consisting only of the discarded
`candidate = (Some First, First)` for its named yielded parameter.

The checked program SHALL retain the nominal Choice identity and ordered First/
Second alternatives inside both recursive fields, the Optional and Result
identities, arithmetic error vocabulary, product arity/order, initial-parameter
yield and suspension, guarded structural-equality action, Unit resumption,
distinct final alternatives, declaration provenance, and ownership edge
separately in all three Generator directions. Application SHALL construct the
initial recursive product once. Traversal SHALL pass that same immutable value
to the action, separately construct its comparison operand, compare Optional
and Result tags before loading their boxed Choice tags, resume with Unit, and
only then construct the final recursive product. This order SHALL hold at LLVM
O0 without relying on folding, inlining, or dead-code elimination.

On Linux x86-64, each admitted Choice payload inside Optional or Result SHALL
use a Topal-owned aligned four-byte box composed with the existing tagged
pointer header; six boxes SHALL be constructed across input, action, and final
values. The product SHALL remain a private two-pointer aggregate and the root-
local Generator a compiler-private `i32` ownership token. LLVM SHALL derive
placement, alignment, and call lowering from the target triple and data layout;
the compiler SHALL hard-code no AMD64 register convention. Two aligned debug-
only aggregate shadows SHALL preserve the yielded action value and captured
initial lifetime. DWARF and GDB SHALL expose the complete recursive Generator
classifier and value, ordered product members, Optional and Result classifiers,
the nested Choice identity and alternatives, yielded/captured values, ordered
yield/action/resumption/final source locations, and the Topal entry frame.

A None or error value, mixed alternative, another Enum declaration or order,
recursive field classifier, product arity/order, error vocabulary, input,
action, final, direction, yield, or value; multiple yields; additional body
statements; close handling; nested or ordinary-function construction;
Generator parameter/result transfer; repeated consumption; abandonment;
libraries; and external boundaries SHALL remain unsupported. The lowering
SHALL introduce no semantic Generator object or state allocation, generic
recursive/Optional/Result dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator/product/
Optional/Result/Choice ABI, or native-ABI revision. Enum boxing, Optional and
Result construction/equality, display, and Linux syscalls SHALL remain wholly
owned by Topal. Future compiled-library metadata SHALL encode recursive nominal
identity, ordered alternatives, error vocabularies, product arity/order and
field identities, independent directions, success/absence evidence, equality
and ordered yield/action/resume/final provenance, allocation-failure effects,
declaration/construction sites, captures/effects, ownership/consumption/close
state, native-representation identity, and target adapters rather than expose
private tokens, debug slots, object layouts, or checked-program nodes.

### TOPAL-COMPILER-GENERATOR-LOCAL-FUNCTION-001 — Retained local declarations

The compiler SHALL admit the shared `custom-generator-local-function.t`
regression: a root `Generator Boolean Unit String` whose body declares the
local enum `Choice is Enum (Accepted, Rejected)`, declares the local ordinary
function `label (value : Choice) -> String` with the complete mapping Accepted
to `"accepted"` and Rejected to `"rejected"`, yields its named Boolean initial
parameter once, and calls `label Accepted` after Unit resumption. The fresh
generator SHALL be consumed exactly once by the existing discarded `not value`
foreach action.

The checked program SHALL retain the local Choice identity and ordered
alternatives, the local function signature and checked Enum decision, the
initial-parameter yield, the post-resume call and argument, declaration and
call provenance, and the generator ownership edge as separate facts. The
local declarations SHALL be available to the resumed continuation but SHALL
not enter the consumer or root name environment. Application SHALL evaluate
the initial Boolean once. Traversal SHALL deliver it to the action, resume with
Unit, directly call the retained function with Accepted, and only then display
the returned String. This order SHALL hold at LLVM O0 without relying on
folding, inlining, or dead-code elimination.

On Linux x86-64, Choice SHALL use its private declaration-ordered `i32` tag and
the root-local Generator SHALL remain a compiler-private `i32` ownership token.
The local function SHALL be emitted as an internal, non-inlined `fastcc`
definition and invoked by a direct call. LLVM SHALL derive machine argument,
return, stack, and register placement from the target triple and data layout;
the compiler SHALL hard-code no AMD64 calling convention placement. The
function and its Choice parameter SHALL have a DWARF subprogram, nominal enum
type and source location. GDB SHALL expose the Generator and Boolean values,
ordered yield/action/resumption/call locations, the `label` frame, its Accepted
argument, and the Topal entry frame.

Another local enum name, alternative or order; another function name,
signature, decision, matcher, action, argument or final call; captures beyond
this exact lexical declaration state; another input, action, direction, yield
or result; multiple yields; additional body statements; close handling;
Generator parameter/result transfer; repeated consumption; abandonment;
libraries; and external boundaries SHALL remain unsupported. The lowering
SHALL introduce no semantic Generator object or state allocation, closure
object, environment allocation, dispatcher, callback, indirect call, unwind
dependency, C/C++ runtime, other-language standard library, needed library,
dynamic relocation, public/library calling convention or Generator/function/
Choice ABI, or native-ABI revision. Enum selection, String allocation/display,
and Linux syscalls SHALL remain wholly owned by Topal. Future compiled-library
metadata SHALL encode lexical declaration identity, parent scope, nominal
alternatives, function signature and checked body graph, capture set,
suspension reachability, direct-call and yield/action/resume/final provenance,
effects, ownership/consumption/close state, native-representation identity, and
target adapters rather than expose private symbols, tags, tokens, debug slots,
object layouts, or checked-program nodes.

### TOPAL-COMPILER-GENERATOR-LOCAL-CLOSE-001 — Restored declarations on close

The compiler SHALL admit the shared
`custom-generator-local-close-handler.t` regression: a root
`Generator Character Unit Unit` whose body declares local
`CloseChoice is Enum (Closed, Continued)`, declares the local ordinary
`cleanup (choice : CloseChoice) -> Unit` with a Unit body, binds
`resume-result is yield initial`, and decides that Result with the ordered
qualified `generator-closed`, fallback Error, and Ok branches from the shared
source. One ordinary `abandon` function SHALL construct a fresh instance and
close it by reaching function-scope Unit without consuming the yield.

The checked program SHALL retain the local nominal identity and ordered
alternatives, local function signature and checked Unit body, yield-result
binding and complete ordered decision, direct cleanup calls and arguments,
declaration and call provenance, and generator ownership/close edge as
separate facts. Close delivery SHALL restore the generator-local declaration
state active at suspension, select only the qualified close branch, directly
call `cleanup Closed`, complete the generator, and only then return from
`abandon`. The local declarations SHALL NOT enter the consumer or root name
environment. This order SHALL hold at LLVM O0 without relying on folding,
inlining, or dead-code elimination.

On Linux x86-64, CloseChoice and the function-local Generator SHALL use
compiler-private declaration-ordered `i32` values. `cleanup` SHALL be emitted
as an internal, non-inlined `fastcc` definition and invoked by a direct call
after Topal-owned Result payload and error-code selection. A target-aligned
debugger-only shadow SHALL preserve its otherwise-unused enum parameter. LLVM
SHALL derive machine argument, return, stack, alignment, and register placement
from the target triple and data layout; the compiler SHALL hard-code no AMD64
calling-convention placement. DWARF and GDB SHALL expose the close Result and
code, the local function subprogram, its complete nominal CloseChoice parameter
and Closed value, ordered close/call locations, and the cleanup, abandon, and
Topal entry frames.

Another local declaration, enum name, alternative or order; another function
name, signature, body, capture, call or argument; another yield, Result subject,
branch, matcher, order or action; another generator direction, result or
ownership path; successful traversal; multiple yields or owned generators;
Generator parameter/result transfer; libraries; and external boundaries SHALL
remain unsupported. The lowering SHALL introduce no semantic Generator object
or state allocation, closure object, environment allocation, close dispatcher,
callback, indirect call, unwind dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library calling
convention or Generator/function/Choice ABI, or native-ABI revision. Future
compiled-library metadata SHALL encode lexical declaration identity and parent
scope, nominal alternatives, function signature and checked body graph,
capture set, suspension and close reachability, ordered Result matchers and
fallback, direct-call and close-site provenance, lexical domain, effects,
ownership/consumption state, native-representation identity, and target
adapters rather than expose private symbols, tags, tokens, debug slots, object
layouts, or checked-program nodes.

### TOPAL-COMPILER-GENERATOR-OVERLOAD-001 — Ordered overload selection and result binding

The compiler SHALL admit the shared `custom-generator-overloads.t` regression
with source-ordered `select (Int)` and `select (Int, String)` declarations.
Application SHALL evaluate its argument exactly once, flatten one unlabeled
positional product for a multi-input candidate, select the first declaration
whose complete ordered input classifiers accept the argument, and bind every
selected operand in declaration order. The admitted unary and binary calls
SHALL use exact `7` and `(7, "item")` inputs. A duplicate complete input signature
SHALL be rejected independently of yield, resume, result, or body differences.

The unary instance SHALL yield its captured Int, execute the admitted Int
action, resume with Unit, and bind its final String. The binary instance SHALL
execute its admitted Int prefix, yield its captured String suffix, execute the
admitted String action, resume with Unit, and bind its final String. A foreach
result classifier SHALL match the selected generator result classifier, and
the fresh result binding SHALL become available only after traversal completes.
The compiler SHALL retain declaration order, ordered parameters and captures,
independent directions, body/suspension graph, final-result provenance, and
the ownership edge as separate checked facts. The generated O0 instruction
order SHALL NOT depend on an LLVM optimization.

On Linux x86-64, overload selection and capture vectors SHALL remain compiler
data. Generator values SHALL keep the private `i32` ownership token; admitted
Int and String values SHALL keep their Topal-owned pointer representations.
LLVM SHALL derive placement and alignment from the target triple and data
layout. DWARF/GDB SHALL expose both selected Generator classifiers, ordered
binary inputs, yielded action values, typed final-result bindings, and source
frames through target-aligned debug-only shadows.

Other overload arities, classifiers, input values, packages, bodies, directions, actions,
results, nested/function construction, transfer, close paths, libraries, and
external boundaries SHALL remain unsupported. The lowering SHALL introduce no
semantic Generator object/state allocation, runtime overload dispatcher,
callback, indirect call, unwind dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library calling
convention or Generator ABI, or native-ABI revision. Future compiled-library
metadata SHALL encode source declaration order, complete ordered input
classifiers and packaging, independent directions, body/suspension graphs,
captures, effects, ownership/consumption/close state, typed result provenance,
native-representation identity, and target adapters rather than private tokens,
debug slots, object layouts, or checked-program nodes.

### TOPAL-COMPILER-GENERATOR-FUNCTION-BOUNDARY-001 — Specialized scalar continuation transfer

The compiler SHALL admit the shared
`custom-generator-generic-function-boundaries.t` regression. The admitted
`numbers` declaration SHALL have the exact classifier
`Generator Int Unit String`, yield its Int input once, resume with Unit, and
return String `"done"`. The ordinary `make` function SHALL accept one Int and
return a fresh instance of that continuation without closing it. The ordinary
`consume` function SHALL accept sole ownership of that continuation, traverse
it exactly once with the admitted Int increment action, bind its final String,
and return that String. The root call SHALL pass exact Int `7` through `make`,
bind the returned continuation, transfer it to `consume`, and produce
`"done"`.

The checked program SHALL retain the complete Generator classifier,
declaration and construction provenance, exact specialized input, suspension
and final-result graph, function-result transfer, function-parameter transfer,
and consumption edge independently of the private machine token. The caller
binding SHALL be consumed by the parameter transfer. Factory exit and consumer
entry SHALL NOT deliver close. The consumer action, Unit resumption, final
String construction, and return SHALL remain source ordered at LLVM O0 and
SHALL NOT depend on inlining, constant folding, or dead-code elimination.

On Linux x86-64, the specialized factory SHALL use a private target-lowered Int
pointer parameter and `i32` Generator-token result, while the consumer SHALL use
the matching private token parameter and Topal String pointer result. LLVM
`fastcc`, the target triple, and the target data layout SHALL determine physical
placement and alignment; the compiler SHALL hard-code no System V register or
return placement. DWARF/GDB SHALL expose the factory Int, the complete Generator
parameter and value, yielded Int action value, final String binding, and both
ordinary-function and Topal entry frames.

Other Generator classifiers, inputs, declarations, factory/consumer shapes,
actions, results, arities, recursion, nesting, repeated use, abandonment,
close paths, packages, libraries, and external boundaries SHALL remain
unsupported. The lowering SHALL introduce no semantic Generator object/state
allocation, runtime transfer or traversal dispatcher, callback, indirect call,
unwind dependency, C/C++ runtime, other-language standard library, needed
library, dynamic relocation, public/library calling convention or Generator
ABI, or native-ABI revision. Future compiled-library metadata SHALL encode the
canonical complete classifier and directions, declaration/body/suspension
graph, construction and both transfer sites, exact or symbolic captures,
effects, ownership/consumption/close state, final-result provenance,
native-representation identity, and target adapters rather than compiler-session
specializations, private tokens, debug slots, pointer layouts, or checked nodes.

### TOPAL-COMPILER-GENERATOR-COMPOUND-FUNCTION-BOUNDARY-001 — Specialized positional-product continuation transfer

The compiler SHALL admit the shared
`custom-generator-compound-function-boundaries.t` regression. The admitted
`pairs` declaration SHALL have the exact classifier
`Generator (Int, String) Unit (Int, String)`, yield its input once, resume with
Unit, and return `(8, "done")`. The ordinary `make` function SHALL accept one
`(Int, String)` and return a fresh instance of that continuation without
closing it. The ordinary `consume` function SHALL accept sole ownership,
traverse it exactly once with the admitted product-equality action, bind its
final product, and return that product. The root call SHALL pass exact
`(7, "item")` through `make`, bind and transfer the returned continuation to
`consume`, and produce `(8, "done")`.

The checked program SHALL retain positional-product arity, source field order,
field classifiers in every Generator direction, declaration and construction
provenance, exact specialized input, suspension, field-wise action,
final-result graph, function-result transfer, function-parameter transfer, and
consumption edge independently of the private machine token. The caller
aggregate SHALL be evaluated once and substituted for the factory-local input
in transferred provenance without reevaluation. Factory exit and consumer
entry SHALL NOT deliver close. Int equality, String equality, Unit resumption,
final-product construction, and return SHALL remain source ordered at LLVM O0
and SHALL NOT depend on inlining, constant folding, or dead-code elimination.

On Linux x86-64, the specialized factory SHALL use the compiler-private LLVM
aggregate of Topal Int and String pointers as its parameter and return an `i32`
Generator token. The consumer SHALL accept that token and return the aggregate.
LLVM `fastcc`, the target triple, and the target data layout SHALL determine
physical aggregate placement, alignment, and return lowering; the compiler
SHALL hard-code no System V register or return placement. DWARF/GDB SHALL expose
the factory product, complete Generator parameter and value, yielded product,
final product, ordered `_0` and `_1` fields, and both ordinary-function and
Topal entry frames.

Other product shapes, field classifiers or order, inputs, declarations,
factory/consumer shapes, actions, results, arities, recursion, nesting,
repeated use, abandonment, close paths, packages, libraries, and external
boundaries SHALL remain unsupported. The lowering SHALL introduce no semantic
Generator object/state allocation, runtime transfer or traversal dispatcher,
callback, indirect call, unwind dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library calling
convention, Generator/product ABI, or native-ABI revision. Future
compiled-library metadata SHALL encode canonical positional-product arity,
order and field identities, complete Generator directions,
declaration/body/suspension graphs, construction and both transfer sites, exact
or symbolic captures, field-wise operations, effects,
ownership/consumption/close state, final-result provenance,
native-representation identity, and target adapters rather than
compiler-session specializations, private tokens, debug slots, aggregate
layouts, or checked nodes.

### TOPAL-COMPILER-GENERATOR-NESTED-FUNCTION-BOUNDARY-001 — Specialized recursive-value continuation transfer

The compiler SHALL admit the shared
`custom-generator-nested-function-boundaries.t` regression. The admitted
`pairs` declaration SHALL have the exact classifier
`Generator Optional (Int, String) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)`.
It SHALL yield its initial Some product once, resume with Unit, and return the
successful product `(8, "done")`. The ordinary `make` function SHALL accept
one `Optional (Int, String)` and return a fresh instance of that continuation
without closing it. The ordinary `consume` function SHALL accept sole
ownership, traverse it exactly once with the admitted Optional-product equality
action, bind its final Result, and return that Result. The root call SHALL pass
exact `Some (7, "item")` through `make`, bind and transfer the continuation to
`consume`, and produce `(8, "done")`.

The checked program SHALL retain the nominal Optional and Result identities,
arithmetic-error vocabulary, success evidence, positional-product arity and
field order, independent Generator directions, declaration and construction
provenance, exact specialized input, suspension, tag-gated field-wise action,
final-result graph, both function-transfer edges, and consumption edge. The
caller Optional SHALL be evaluated once and substituted for the factory-local
input in transferred provenance without reevaluation. Factory exit and
consumer entry SHALL NOT deliver close. Optional tag checks, Int and String
equality, Unit resumption, successful Result construction, and return SHALL
remain source ordered at LLVM O0 and SHALL NOT depend on inlining, constant
folding, or dead-code elimination.

On Linux x86-64, the specialized factory SHALL use the existing Topal-owned
Optional pointer parameter and return an `i32` Generator token. The consumer
SHALL accept that token and return the existing Topal-owned Result pointer;
successful product payloads SHALL retain aligned storage for the existing Int
and String pointers. LLVM `fastcc`, the target triple, and target data layout
SHALL determine physical placement and alignment; the compiler SHALL hard-code
no System V register or return placement. A target-aligned factory parameter
shadow SHALL retain the otherwise compile-time-only argument lifetime.
DWARF/GDB SHALL expose the complete Generator classifier, Optional and Result
product classifiers, arithmetic error vocabulary, initial/yield/final values,
ordinary-function frames, and Topal entry frame.

Other nested classifiers, alternatives, product shapes, inputs, declarations,
factory/consumer shapes, actions, results, arities, recursion, nesting,
repeated use, abandonment, close paths, packages, libraries, and external
boundaries SHALL remain unsupported. The lowering SHALL introduce no semantic
Generator object/state allocation, runtime transfer or traversal dispatcher,
callback, indirect call, unwind dependency, C/C++ runtime, other-language
standard library, needed library, dynamic relocation, public/library calling
convention, Generator/Optional/Result/product ABI, or native-ABI revision.
Future compiled-library metadata SHALL encode nominal wrapper identities and
alternatives, error vocabulary, recursive product arity/order/field identities,
complete Generator directions, declaration/body/suspension graphs,
construction and both transfer sites, exact or symbolic captures, tag and
field-operation provenance, effects, ownership/consumption/close state,
final-result evidence, native-representation identity, and target adapters
rather than compiler-session specializations, private tokens, debug slots,
object layouts, or checked nodes.

### TOPAL-COMPILER-GENERATOR-LIST-FUNCTION-BOUNDARY-001 — Specialized List continuation transfer

The compiler SHALL admit the shared `custom-generator-list-values.t`
regression. The admitted `relay` declaration SHALL have the exact classifier
`Generator List Int Unit List Int`. It SHALL yield its initial List once,
resume with Unit, and return `initial append 9`. The ordinary `make` function
SHALL accept one `List Int` and return a fresh instance of that continuation
without closing it. The ordinary `consume` function SHALL accept sole
ownership, traverse it exactly once with the discarded `entry-count values`
action, bind its final List, and return that List. The root call SHALL pass
exact `one 7` through `make`, bind and transfer the continuation to `consume`,
and produce `Entry ( 7, Entry ( 9, Empty ) )`.

The checked program SHALL retain the recursive List identity, Int element
classifier, immutable node order, complete Generator directions, declaration
and construction provenance, exact specialized input, suspension, action,
final append graph, both function-transfer edges, and consumption edge. The
caller List SHALL be evaluated once and substituted for the factory-local
input in transferred provenance without reevaluation. Factory exit and
consumer entry SHALL NOT deliver close. The entry-count action SHALL complete
before Unit resumption; only afterward SHALL the final singleton be allocated,
appended through the existing immutable List operation, and returned. This
order SHALL hold at LLVM O0 without inlining, folding, or dead-code elimination.

On Linux x86-64, the specialized factory SHALL use the existing Topal-owned
private `List Int` pointer parameter and return an `i32` Generator token. The
consumer SHALL accept that token and return the existing private List pointer.
LLVM `fastcc`, the target triple, and target data layout SHALL determine
physical placement and alignment; the compiler SHALL hard-code no System V
register or return placement. A target-aligned factory parameter shadow and
List action/final-lifetime shadows SHALL retain otherwise compile-time-only
values. DWARF/GDB SHALL expose the complete Generator classifier, initial,
yielded and final Lists, ordinary-function frames, and Topal entry frame.

Other List element classifiers, inputs, declarations, factory/consumer shapes,
actions, final operations or values, yields, directions, arities, recursion,
nesting, repeated use, abandonment, close paths, packages, libraries, and
external boundaries SHALL remain unsupported. The lowering SHALL introduce no
semantic Generator object/state allocation, runtime transfer or traversal
dispatcher, callback, indirect call, unwind dependency, C/C++ runtime,
other-language standard library, foreign allocator, needed library, dynamic
relocation, public/library calling convention, Generator/List ABI, or native-
ABI revision. Future compiled-library metadata SHALL encode recursive nominal
List identity, element classifier, immutable operation and allocation effects,
complete Generator directions, declaration/body/suspension graph,
construction and both transfer sites, exact or symbolic captures,
action/final-operation provenance, ownership/consumption/close state,
native-representation identity, and target adapters rather than private
tokens, node pointers/layouts, debug slots, compiler specializations, or
checked nodes.

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
above; labeled products; additional body statements; close handling; nested
construction; ordinary-function construction or Generator parameter/result
transfer outside
`TOPAL-COMPILER-GENERATOR-COMPOUND-FUNCTION-BOUNDARY-001`; repeated
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

Direct `root member` data access from a compiled function body is governed by
`TOPAL-COMPILER-FUNCTION-ROOT-DATA-001`. Scope results/escape, nested qualified
Scope members, generators, `use`, packages, and source or compiled libraries
remain outside this increment and SHALL be rejected until a storage/interface
representation valid beyond the source entry frame exists.

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
and compiled-library Scope environments remain deferred. Exact root and
root-alias package fields are governed by
`TOPAL-COMPILER-SCOPE-PACKAGED-FIELD-001`; all other Scope package forms remain
deferred.

### TOPAL-COMPILER-NAMESPACE-GENERATOR-001 — Static qualified generator application

At source root, the live `root` Scope and every retained root alias SHALL carry
the generator declarations visible in that namespace snapshot, including each
complete source-ordered overload set and its namespace provenance. A generator
introduced after an alias binding SHALL NOT enter that alias. It SHALL remain
available through a later live-root selection. A same-named lexical declaration
SHALL NOT intercept or join a qualified generator selection.

Applying `root member operands` or `alias member operands` SHALL select only
from that namespace's retained generator declarations before applying the
ordinary checked generator rules. Each operand SHALL be evaluated exactly once.
Every application SHALL produce a fresh affine continuation with the same
suspension, resumption, result, close, consumption, and abandonment obligations
as the corresponding unqualified application. Every custom-generator graph
otherwise admitted by the compiler SHALL reuse its existing checked graph and
O0 inline traversal; namespace provenance SHALL NOT change source-visible
Generator behavior.

Namespace and generator selection SHALL complete before LLVM lowering. Scope,
Generator, yielded values, and source location SHALL remain observable in
DWARF/GDB, but the implementation SHALL require no runtime namespace lookup or
table, continuation object, callback, indirect dispatch, foreign dependency,
C/C++ runtime, other-language standard library, public generator ABI, or native
ABI revision.

A future separately compiled interface for a published generator member SHALL
identify the namespace and interface revision, member visibility, overload
signature, checked directions and body graph, capture and effect facts,
linearity and ownership rules, suspension and close behavior, semantic native
representation, and versioned target adapter. Compiler-private tags and symbols
SHALL NOT serve as portable semantic identity. Non-root or external namespaces,
generator-bearing Scope parameters/results, dynamic or escaping qualified
generator values, generator shapes not otherwise admitted by the compiler,
packages, and source or compiled libraries remain outside this increment.

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

### TOPAL-COMPILER-SYMBOLIC-CALLABLE-EXPANDED-001 — Complete symbolic Function values

In addition to `+`, `-`, and `<=>`, the symbolic callables `=`, `!=`, `<`, `>`,
`<=`, `>=`, `*`, `/`, `/%`, `%`, `^`, `..`, `<..`, `..=`, and `<..=` in value
position SHALL produce Function values retaining their exact callable identity.
Each application SHALL require one two-field positional product and SHALL reuse
the corresponding existing checked operation, including its conversions,
fallibility, result classifier, endpoint policy, and diagnostic behavior. These
values MAY cross the private specialized Function parameter and result paths
admitted by `TOPAL-COMPILER-FUNCTION-PARAMETER-001` and
`TOPAL-COMPILER-FUNCTION-RESULT-001`.

The deterministic private Function observation table SHALL retain the existing
`+`, `-`, and `<=>` tag order and append the newly admitted canonical spellings.
The source inequality spelling `!=` SHALL be observed canonically as `/=` in
Function display and DWARF/GDB. Checked callable metadata, rather than the tag,
SHALL select the ordinary direct LLVM operation or Topal-owned runtime primitive.

This rule SHALL introduce no function pointer, indirect call, Function runtime,
closure allocation, foreign dependency, C/C++ runtime, other-language standard
library, public callable ABI, or native-ABI revision. Dynamically selected,
aggregate-contained Function values outside
`TOPAL-COMPILER-FUNCTION-AGGREGATE-001`, escaping values, and published callable
interfaces remain deferred.

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

Function results and Function values embedded in aggregate boundaries outside
their dedicated rules, anonymous functions/captures, remaining symbolic
callables, and published callable interfaces remain outside this increment and
SHALL be rejected.

### TOPAL-COMPILER-FUNCTION-RESULT-001 — Specialized private Function results

An ordinary or static root function whose declared result classifier is
`Function` SHALL admit an already-supported named root or symbolic Function
value as its result. A specialized `Function` parameter MAY be returned when
its call-site identity is one of those admitted values. The checked compiler
SHALL retain the result's callable identity through the direct call and any
subsequent binding chain; applying the result SHALL use that retained identity
and SHALL NOT dispatch on its machine value or restart name lookup.

The exact private LLVM signature SHALL return the existing i32 Function
observation tag. The caller SHALL issue a direct `fastcc` call to obtain that
tag, while later application SHALL lower directly to the selected private
function or operation. At O0, Function-result locals SHALL use target-aligned
debug-only stack shadows so DWARF/GDB can observe the returned value even when
specialization makes the tag computationally dead. LLVM SHALL own the physical
AMD64 register and stack placement.

Capturing anonymous results are governed by
`TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001`; exact aggregate containment is
governed by `TOPAL-COMPILER-FUNCTION-AGGREGATE-001` and
`TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`. Nested, dynamically computed,
and published Function results outside those rules remain rejected.
No function pointer, indirect call, closure allocation/runtime, foreign
dependency, C/C++ runtime, other-language standard library, public callable
ABI, or native ABI revision is permitted.

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
callable interfaces are not admitted by this rule alone; unsupported forms
SHALL be rejected rather than referencing storage from another call frame.

### TOPAL-COMPILER-ANONYMOUS-CAPTURE-001 — Private anonymous captures and results

A bound inferred anonymous function MAY capture immutable lexical data with an
admitted complete private representation when every application remains in the
same defining invocation. The checked compiler SHALL retain the construction-
time storage identities and SHALL pass their already-evaluated values after the
explicit source operands in deterministic lexical-name order. The specialized
anonymous body SHALL resolve each captured name only to its corresponding
hidden parameter; caller shadowing and later bindings SHALL NOT participate.

A non-capturing inferred anonymous function MAY be the exact `Function` result
of an ordinary or static root function. The caller SHALL retain that anonymous
body, arity, construction identity, and empty capture set through the direct
result call and later bindings. Application SHALL specialize the anonymous body
and issue a direct private call; it SHALL NOT dispatch through the returned
machine tag or repeat callable lookup.

The capture-specialized LLVM signature SHALL list exact source parameters
followed by exact capture parameters and SHALL use private `fastcc`, leaving
physical AMD64 placement to LLVM. A non-capturing anonymous result SHALL use the
existing private i32 Function observation tag and debug-only result shadow.
DWARF/GDB SHALL expose explicit parameters, material captures under their source
names, the anonymous source frame, and returned Function locals at `-O0`.

A capturing anonymous value crosses a private Function parameter only under
`TOPAL-COMPILER-FUNCTION-CAPTURE-PARAMETER-001` and crosses an admitted private
result only under `TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001`. Captures without
an admitted private representation, other dynamic escape, capture-bearing
aggregate boundaries, publication, and library boundaries SHALL be rejected
before LLVM lowering. This rule SHALL introduce no environment object, function
pointer, indirect call, closure or Function runtime, foreign dependency, C/C++
runtime, other-language standard library, public callable ABI, or native-ABI
revision.

### TOPAL-COMPILER-FUNCTION-CAPTURE-PARAMETER-001 — Private captured Function parameters

A capturing anonymous Function or non-escaping nested lexical Function MAY be
passed through an ordinary private `Function` parameter when the complete
callable identity and every captured immutable value remain known to the
checked compiler. The capture set SHALL contain only values with already-
admitted exact private function-boundary representations and SHALL contain no
Generator or Function. Construction SHALL snapshot each value once. Every
specialized caller SHALL forward those same values, without re-evaluation, in a
deterministic retained order after the source parameters. Forwarding MAY cross
multiple private ordinary-function calls while the values remain within their
defining lifetime.

The checked callee SHALL retain the Function observation tag separately from
its callable body and capture metadata. Applying the parameter SHALL specialize
that retained body and issue one direct private call with its explicit operands
followed by the forwarded captures. A nested lexical Function MAY acquire its
module-private observation tag for Function-parameter passage, but that tag
SHALL identify display/debug state only and SHALL NOT dispatch the call.

Every generated definition and call SHALL use an exact private `fastcc`
prototype; LLVM SHALL derive physical aggregate and scalar placement from the
target data layout. DWARF SHALL expose the source Function parameter and the
material captures under their source names in the invoked anonymous or nested
frame. Pure forwarding parameters that have no corresponding source binding
SHALL remain absent from DWARF. Tests SHALL cover multiple scalar captures, an
aggregate capture, root and function-local construction, transitive forwarding,
a non-escaping nested Function, exact interpreter parity, and full O0 GDB
frames.

This rule SHALL introduce no environment object or allocation, function
pointer, indirect call, callback, closure or Function runtime, foreign
dependency, C/C++ runtime, other-language standard library, public callable
ABI, or native-ABI revision. Capturing Function results outside
`TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001`, capture-bearing aggregate
containment outside `TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`, dynamic
selection or escape, recursive or overloaded nested callable values,
unsupported captured state, publication, and library metadata/adapters remain
deferred. Future compiled-library metadata SHALL
encode callable identity, ordered capture identities and classifiers,
construction lifetime, effects, native-representation identities, and target
adapters rather than expose this private hidden-parameter layout.

### TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001 — Private captured Function results

An ordinary or static private function MAY return a capturing anonymous
Function when the exact anonymous body and every construction-time immutable
capture remain known to the checked compiler. Every capture SHALL have an
already-admitted complete private function-boundary representation and SHALL
contain neither Generator nor Function. A specialized Function parameter with
those same anonymous callable facts MAY be returned. An admitted captured
result MAY subsequently cross another admitted private Function result or
parameter while its returned values remain intact.

The specialized callee SHALL evaluate the source Function result once and
return one exact private aggregate containing the deterministic i32 observation
tag followed by the already-evaluated captures in retained order. The caller
SHALL issue one direct `fastcc` call, decompose the aggregate into compiler-only
SSA storage, and retain the callable facts separately from the tag. Later
application SHALL specialize the retained anonymous body and issue a direct
call with the explicit operands followed by those returned values. The tag
SHALL NOT dispatch application, and capture initializers SHALL NOT be replayed.

LLVM SHALL derive physical scalar and aggregate return placement from the
target data layout. DWARF SHALL describe the factory result and source binding
as `Function`, omit the private returned transport fields, and expose the
material captures under their source names in the eventually invoked anonymous
frame. Tests SHALL cover multiple scalar captures, an aggregate capture, root
and function-local factories, Function-parameter pass-through, transitive
Function-result forwarding, exact interpreter parity, and full O0 GDB frames.

This rule SHALL introduce no heap or environment object, environment pointer,
allocation, function pointer, indirect call, callback, closure or Function
runtime, foreign dependency, C/C++ runtime, other-language standard library,
public callable ABI, or native-ABI revision. Immediate exact result application
is governed by `TOPAL-COMPILER-FUNCTION-RESULT-CHAIN-001`. Exact nested Function
escape is governed by `TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`.
Function-valued or otherwise unsupported captures, dynamic selection,
capture-bearing aggregate containment outside
`TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`, publication, and library
metadata/adapters remain deferred. Future compiled-library metadata SHALL
encode callable identity, ordered capture identities and classifiers,
construction lifetime, effects, native-representation identities, and target
adapters rather than expose this private aggregate transport.

### TOPAL-COMPILER-FUNCTION-RESULT-CHAIN-001 — Exact Function-result application chains

The compiler SHALL implement the left-associative application semantics of
`TOPAL-TYPE-CALL-001` when an admitted application produces an exact Function
value and the next source operand immediately applies that result. Flat prefix
chains and explicitly parenthesized intermediate applications SHALL behave
identically. The admitted result MAY retain a named overload set, symbolic
callable identity, non-capturing anonymous body, or capturing anonymous body
under `TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001`.

Every intermediate application SHALL execute exactly once and in source order.
The checked frontend SHALL retain exact callable facts separately from the
observation tag and SHALL represent the intermediate Function only in
compiler-private storage. A captured result's already-extracted capture values
SHALL remain available to its immediate anonymous specialization. Eventual
application SHALL use an exact direct `fastcc` call; the tag SHALL NOT dispatch
the call. A chain whose intermediate result is not Function or whose operand
does not satisfy the retained callable SHALL be rejected before LLVM lowering.

Tests SHALL cover flat and parenthesized chains, named and symbolic pass-through,
non-capturing and capturing anonymous results, scalar and product operands,
once-only LLVM calls, exact interpreter parity, reversible history, and O0 GDB
source frames. Compiler-only chain storage SHALL NOT appear as a source DWARF
variable. This rule SHALL introduce no allocation, environment object, function
pointer, indirect call, callback, closure or Function runtime, foreign
dependency, C/C++ runtime, other-language standard library, public callable
ABI, or native-ABI revision. Aggregate-contained or dynamically selected
Function values outside `TOPAL-COMPILER-FUNCTION-AGGREGATE-001`, nested escape
outside `TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`, publication, and library
metadata/adapters remain deferred.

### TOPAL-COMPILER-FUNCTION-AGGREGATE-001 — Exact private Function aggregates

A private Tuple or Record MAY recursively contain Function fields when the
checked compiler retains one exact named, symbolic, or anonymous callable
identity for every such field. Construction and binding SHALL evaluate each
source field once in source order and preserve callable identity separately
from its deterministic observation tag. Direct Record field selection and
anonymous positional-product destructuring SHALL recover those facts and apply
the selected callable through its ordinary checked semantics.

The capture-free base case permits an ordinary private function parameter or
result to carry such a Tuple or Record when every Function leaf's exact
callable identity remains known at the call site. The checked frontend SHALL
propagate the complete recursive structural facts across that specialization
boundary. Capture-bearing aggregate boundaries are governed by
`TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`.
A local aggregate MAY contain a capturing anonymous Function while selection
and application remain within the captured values' defining lifetime.
Opaque, branch-selected, or otherwise dynamically computed Function leaves;
and Function containment outside Tuple, Record, the exact Optional path of
`TOPAL-COMPILER-OPTIONAL-FUNCTION-001`, or the exact nominal Sum path of
`TOPAL-COMPILER-SUM-FUNCTION-001`, or the exact Result-success path of
`TOPAL-COMPILER-RESULT-FUNCTION-001`, or the exact finite List-entry paths of
`TOPAL-COMPILER-LIST-FUNCTION-001`, or the exact fixed-size Array-entry paths of
`TOPAL-COMPILER-ARRAY-FUNCTION-001`, or the exact String-keyed Map-value paths
of `TOPAL-COMPILER-MAP-FUNCTION-001` SHALL be rejected before LLVM lowering.

Each represented Function leaf SHALL occupy its existing private i32
observation field. Exact private `fastcc` prototypes SHALL use the corresponding
recursive LLVM aggregate type, and LLVM SHALL derive physical x86-64 argument
and result placement from the qualified target data layout. Eventual
application SHALL remain a direct specialized call selected from checked
metadata; the observation field SHALL NOT dispatch control flow. DWARF/GDB
SHALL expose source Tuple/Record parameters, results, and locals with embedded
Function identities and recursively accurate field types.

Tests SHALL cover local captured containment; named, symbolic, and non-capturing
anonymous values; recursive Tuple/Record construction; parameter and result
passage; field selection and destructuring; exact direct IR; interpreter parity
and reversible history; freestanding execution; and full O0 GDB values/frames.
This rule SHALL add no closure object, environment pointer, allocation,
function pointer, indirect call, callback, dispatch table, foreign dependency,
C/C++ runtime, other-language standard library, public aggregate/callable ABI,
or native-ABI revision. Dynamic aggregate boundaries, capture-bearing
boundaries outside `TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`, other
aggregate constructors, escape, publication, and library adapters remain
deferred. Future compiled-library metadata SHALL encode every Function field's
canonical aggregate path, callable identity, ordered captures and classifiers,
construction lifetime, effects, representation identity, and target adapter
independently of private LLVM types and observation tags.

### TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001 — Captured private Function aggregates

An ordinary private function parameter MAY carry an exact recursively nested
Tuple or Record whose Function leaves are capturing anonymous Functions or
nested Functions. An ordinary private function result MAY carry capturing
anonymous Function leaves and exact nested Function leaves under
`TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`. Every leaf SHALL retain one exact
callable identity, its complete ordered capture facts, and its canonical
aggregate path.
Canonical paths SHALL consist of zero-based Tuple indexes, Record labels,
admitted Optional payload edges, admitted nominal Sum alternative-name payload
edges, admitted Result-success edges, zero-based List and Array entry edges,
and semantic exact String-keyed Map-value edges. They SHALL be ordered by a
depth-first, left-to-right traversal of the source aggregate; Map edges SHALL
use first-key occurrence order after collision resolution. Optional, Sum,
Result, List, Array, and Map edges SHALL satisfy their respective exact-container
rules.

The checked frontend SHALL append one hidden capture operand for every capture
at every Function path after the source-visible aggregate operand. A result
SHALL return the source aggregate followed by the same path-ordered capture
values in a compiler-private aggregate. Calls, bindings, forwarding results,
Record selection, and recursive positional-product destructuring SHALL remap
those hidden values to the selected callable's capture storage without replaying
the aggregate, a field initializer, or a capture initializer. A Function leaf
SHALL be applied only through its retained direct specialization. Capture state
that itself contains Function values, Generator state, an opaque callable, a
dynamically selected leaf, or a value outside the represented private lifetime
SHALL be rejected before LLVM lowering.

Private definitions and calls SHALL use matching direct `fastcc` prototypes.
LLVM SHALL own physical x86-64 register, stack, and aggregate-result placement
from the qualified target data layout. Each source aggregate SHALL retain its
ordinary recursive DWARF type and Function observation fields; hidden capture
transport SHALL NOT appear as an aggregate member or compiler-named source
variable. An eventual anonymous or nested call SHALL expose material captures
under their source names in that callable's frame.

Tests SHALL cover multiple captured Record leaves, a captured Tuple leaf,
parameter-to-result forwarding, recursive destructuring, a nested Function
parameter, exact nested escape under
`TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`, rejected Function-containing
capture state, exact path-ordered direct IR, interpreter parity and reversible
history, freestanding execution, and full O0 GDB aggregate values/capture
frames. This rule SHALL add no closure object, environment pointer, allocation, function
pointer, indirect call, callback, dispatch table, foreign dependency, C/C++
runtime, other-language standard library, public aggregate/callable ABI, or
native-ABI revision. Dynamic Function selection, other aggregate constructors,
escaping nested Functions outside `TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`,
persistent storage, publication, and public library adapters remain deferred.
Future compiled-library metadata SHALL encode the
canonical aggregate path; callable and capture identities; capture classifiers
and order; construction lifetime; effects; representation identity; and target
adapter independently of private LLVM aggregate types, observation tags, and
the current hidden-operand layout.

### TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001 — Exact private nested Function escape

Within one compilation unit, an ordinary private function MAY return an exact
nested named Function directly, in a recursively nested Tuple or labeled
Record, under the exact present Optional path governed by
`TOPAL-COMPILER-OPTIONAL-FUNCTION-001`, or under an exact selected nominal Sum
path governed by `TOPAL-COMPILER-SUM-FUNCTION-001`, or under the success path
of an exact arithmetic Result governed by
`TOPAL-COMPILER-RESULT-FUNCTION-001`, or under exact finite List-entry paths
governed by `TOPAL-COMPILER-LIST-FUNCTION-001`, or under exact fixed-size
Array-entry paths governed by `TOPAL-COMPILER-ARRAY-FUNCTION-001`, or under
exact String-keyed Map-value paths governed by
`TOPAL-COMPILER-MAP-FUNCTION-001`. The nested
declaration SHALL be one
nonrecursive, nonoverloaded ordinary declaration whose identity remains known
at every private boundary.
Each captured value SHALL be an already-evaluated immutable lexical,
defining-context, or live-root value with a complete admitted private
representation; capture state SHALL contain neither Function nor Generator.
Separate invocations of the same factory SHALL retain separate capture
snapshots even though the nested declaration has one observation tag.

A scalar result SHALL use the exact private Function-result aggregate of
`TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001`. A Tuple, Record, Optional, Sum,
or Result result SHALL use the canonical Function-leaf path and capture ordering of
`TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`. The caller SHALL decompose the
result once into compiler-only SSA storage and own the returned values after the
factory frame ends. Binding, private parameter/result forwarding, direct field
selection, recursive product destructuring, and immediate left-associative
application MAY remap that storage without replaying the factory, capture
initializers, or aggregate construction.

Every eventual application SHALL specialize the retained nested declaration
and issue one direct private `fastcc` call with explicit operands followed by
the returned capture values. The observation tag SHALL NOT dispatch control
flow. LLVM SHALL derive AMD64 argument and result placement from the qualified
target data layout, and correctness SHALL require no optimization. DWARF SHALL
describe the factory result and binding as `Function`, preserve source
Tuple/Record layouts without hidden transport fields, and expose explicit
parameters and captures under their source names in the invoked nested frame,
including while an intermediate caller is suspended.

Tests SHALL cover distinct invocations of one factory, lexical scalar and
aggregate captures, defining-context and live-root captures, scalar and Record
results, transitive forwarding, immediate result application, exact interpreter
parity and reversible history, direct LLVM IR, freestanding ELF/DWARF, full O0
GDB frames, artifact-free unsupported-capture rejection, the shared corpus, and
separate interpreter/compiler resource baselines.

This rule SHALL add no heap, closure or environment object, environment
pointer, allocation, function pointer, indirect call, callback, dispatcher,
global context/root state, caller-frame lookup, foreign dependency, C/C++
runtime, other-language standard library, public callable ABI, or native-ABI
revision. Recursive, overloaded, sibling-dependent, opaque, or dynamically
selected nested Functions; Function- or Generator-containing capture state;
persistent storage; publication; and public/library escape remain deferred and
SHALL fail before artifact publication. Future compiled-library metadata SHALL
encode canonical source-session, lexical-scope, nested-declaration, callable,
aggregate-path, capture identity/order/classifier/representation/lifetime,
effect, and versioned target-adapter facts independently of observation tags,
compiler-private names, LLVM types or symbols, debug shadows, and physical
placement.

### TOPAL-COMPILER-OPTIONAL-FUNCTION-001 — Exact private Optional Function environments

Within one compilation unit, an exact `Optional Function` MAY be constructed,
bound, displayed, passed through an ordinary private function parameter or
result, and recursively contained in an admitted Tuple or labeled Record. The
checked frontend SHALL retain exact absence separately from a present payload.
For `Some`, it SHALL retain one exact named, symbolic, anonymous, or admitted
nested callable identity independently of the Function observation value and
the Optional tag. Opaque or branch-selected present callable identity SHALL be
rejected before LLVM lowering.

`Some` SHALL use the existing Topal-owned Optional allocation and header and
box only the existing private i32 Function observation value. `None Function`
SHALL have no payload. An exhaustive Optional decision over an exact present
value SHALL attach the retained callable facts to the `Some` payload binding;
eventual application SHALL remain one direct specialized private `fastcc`
call, and neither Optional nor Function tags SHALL dispatch it. Display and
DWARF/GDB SHALL describe `Some <function identity>` or `None` through the
source `Optional Function` classifier without exposing compiler-only facts.

A present capturing callable SHALL use an Optional-payload element in the
canonical path of `TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`. Private
parameter specialization SHALL pass the source Optional pointer followed by
the payload callable's ordered capture values. A private result SHALL return
the unchanged Optional pointer followed by those values; its caller SHALL own
and remap them exactly once. An exact nested Function MAY therefore escape a
factory through `Some` under `TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`.
Function- or Generator-containing capture state SHALL remain unsupported.

A repeated anonymous-pattern name MAY compare two exact Optional Function
values. The guard SHALL compare Optional presence and the boxed Function
observation value first. When both present payloads retain the same callable
identity, it SHALL additionally compare corresponding captures in canonical
order using already-admitted exact compiler equality. A different callable or
presence SHALL mismatch without requiring capture transport from an
unselected identity. No user Equality operation or allocation identity SHALL
participate.

Tests SHALL cover named, symbolic, anonymous, and nested payloads; exact
absence; independent captured factory results; direct `Some` application;
private parameter/result and Tuple/Record containment; repeated identity and
mismatch; exact display; interpreter parity and reversible history; direct
LLVM IR; artifact-free opaque/capture rejection; freestanding ELF/DWARF; full
O0 GDB values and callable frames; the shared corpus; and separate
interpreter/compiler resource baselines.

This rule SHALL add no closure/environment object, environment pointer,
function pointer, indirect call, callback, dispatch table, caller-frame lookup,
foreign dependency, C/C++ runtime, other-language standard library, public
callable/container ABI, or native-ABI revision. Allocation SHALL be limited to
the already-defined process-lifetime Optional representation. Optional
callables with opaque or dynamically selected identity, persistent storage,
publication, and Function containment in containers other than exact nominal
Sums governed by `TOPAL-COMPILER-SUM-FUNCTION-001` remain deferred and
SHALL fail before artifact publication. Future compiled-library metadata SHALL
encode canonical source-session, scope, callable/declaration, Optional payload
and enclosing aggregate paths, presence/selection proof, ordered capture
identity/classifier/representation/lifetime, effects, container representation,
and versioned target adapter independently of private observation tags, header
layout, compiler names, LLVM types/symbols, debug shadows, and physical
placement.

### TOPAL-COMPILER-SUM-FUNCTION-001 — Exact private nominal Sum Function environments

Within one compilation unit, an exact nominal labeled `Union` or positional
`Variant` MAY contain a Function-bearing payload, be constructed, bound,
displayed, passed through an ordinary private parameter or result, used as a
package field, and be recursively contained in an admitted Tuple or labeled
Record. The checked frontend SHALL retain the exact selected alternative and
its complete payload fact tree independently of the runtime Sum tag. Every
active Function leaf SHALL retain one exact named, symbolic, anonymous, or
admitted nested callable identity. Opaque or branch-selected alternatives or
callable identities SHALL reject before LLVM lowering.

The source Sum SHALL retain the exact private tag-plus-declaration-ordered-
payload representation of `TOPAL-COMPILER-SUM-001`. A complete Sum decision
over an exact selected value SHALL attach its retained payload facts to the
selected binding; eventual Function application SHALL remain one direct
specialized private `fastcc` call, and neither tag SHALL dispatch it. Display,
DWARF, and GDB SHALL expose the nominal source classifier and active
alternative/payload without exposing inactive slots or compiler facts.

Each active Function leaf SHALL add an alternative-name payload edge to the
canonical path of `TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`, composed
with enclosing Tuple and Record edges. Private parameters SHALL pass the source
Sum aggregate followed by canonical-path-ordered captures. Private results
SHALL return the unchanged source aggregate followed by those captures, and the
caller SHALL extract and remap them exactly once. An exact nested Function MAY
therefore escape through a selected Sum payload under
`TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`. Function- or Generator-containing
capture state and recursive Sum declarations SHALL remain unsupported.

A repeated anonymous-pattern name MAY compare two exact Function-bearing Sums.
The guard SHALL compare nominally identical tags first and then only the active
payload. Function observation values SHALL be compared before captures. When
both operands retain the same selected alternative and callable identity, the
guard SHALL additionally compare corresponding captures in canonical order
using admitted exact compiler equality. A different tag or callable SHALL
mismatch without observing inactive payloads or requiring an unselected
capture schema. This compiler-private guard SHALL NOT grant general source
Equality to a Function-bearing Sum.

Tests SHALL cover labeled and positional Sums; named, symbolic, anonymous, and
nested Function payloads; payload-free and non-Function alternatives; distinct
captured factory results; direct decision application; private parameter,
result, package, Tuple, and Record passage; repeated match/mismatch; display;
interpreter parity and reversible history; direct LLVM IR; artifact-free
opaque/capture rejection; freestanding ELF/DWARF; full O0 GDB values and
callable frames; the shared corpus; and separate interpreter/compiler resource
baselines.

This rule SHALL add no closure/environment object, environment pointer,
function pointer, indirect call, callback, dispatch table, caller-frame lookup,
allocation, foreign dependency, C/C++ runtime, other-language standard library,
public callable/Sum ABI, or native-ABI revision. Inactive payload slots SHALL
remain a private LLVM representation detail. Opaque or dynamically selected
Sum callables, recursive Sums, persistent storage, publication, and Function
containment in other containers remain deferred. Future compiled-library
metadata SHALL encode canonical source-session/scope and nominal Sum identity,
labeled/positional form, ordered alternatives and payload classifiers, exact
active-alternative proof, callable/declaration and aggregate paths, ordered
capture identity/classifier/representation/lifetime, effects, Sum
representation, and versioned target adapters independently of private tags,
inactive layout, compiler names, LLVM types/symbols, debug shadows, and physical
placement.

### TOPAL-COMPILER-RESULT-FUNCTION-001 — Exact private Result Function environments

Within one compilation unit, an exact
`Result (Function, lang arithmetic ArithmeticErrorCode)` MAY be constructed by
the existing successful-Result contract, bound, displayed, passed through an
ordinary private parameter or result, used as a package field, and recursively
contained in an admitted Tuple or labeled Record. The checked frontend SHALL
retain one exact named, symbolic, anonymous, or admitted nested callable
identity and its complete capture facts conditionally for the success payload.
The runtime success/Error tag MAY remain dynamic. An opaque or branch-selected
success callable SHALL reject before LLVM lowering.

The source Result SHALL retain the exact Topal-owned pointer representation of
`TOPAL-COMPILER-RESULT-001`. Success SHALL box only the existing private i32
Function observation value. Failure and contextual projection SHALL preserve
the complete structured Error unchanged, including domain, code, detail,
cause, and source provenance. A complete Result decision SHALL attach retained
callable facts only to the `Ok` binding; eventual application SHALL remain one
direct specialized private `fastcc` call, and neither the Result tag nor the
Function observation SHALL dispatch it. Display, DWARF, and GDB SHALL expose
the source Result classifier and selected success/Error value without exposing
compiler facts or hidden capture transport.

Each successful Function payload SHALL add a Result-success edge to the
canonical path of `TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`, composed
with enclosing Tuple and Record edges. Private parameters SHALL pass the source
Result pointer followed by canonical-path-ordered captures. Private results
SHALL return the unchanged pointer followed by those captures. Every Error
exit from such a function SHALL return the same private aggregate shape and
place representation-valid zero carriers in hidden capture fields without
evaluating skipped Function or capture initializers. A caller SHALL attach
those fields only to the conditional success facts, and an Error path SHALL
never observe or apply them. An exact nested Function MAY therefore escape
through success under `TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`.

Tests SHALL cover named, symbolic, anonymous, and nested success payloads; an
original dynamically propagated Error; distinct captured factory results;
direct `Ok` decision application and `Error` handling; private parameter,
result, package, Tuple, and Record passage; exact display; interpreter parity
and reversible history; direct LLVM IR including aggregate Error exits;
artifact-free opaque/capture rejection; freestanding ELF/DWARF; full O0 GDB
Result/callable values and active/suspended frames; the shared corpus; and
separate interpreter/compiler resource baselines.

This rule SHALL add no closure/environment object, environment pointer,
function pointer, indirect call, callback, dispatch table, caller-frame lookup,
foreign dependency, C/C++ runtime, other-language standard library, public
callable/Result ABI, or native-ABI revision. Allocation SHALL be limited to the
existing process-lifetime Result representation and successful i32 payload box.
Function- or Generator-containing capture state, opaque or dynamically
selected success callables, general Result Function equality or repeated
identity, persistent storage, publication, recursive Result Function payloads,
and Function containment in other containers remain deferred. Future
compiled-library metadata SHALL encode the canonical source-session/scope,
Result success classifier and Error-code vocabulary, conditional-success
proof, Error propagation semantics, callable/declaration and aggregate paths,
ordered capture identity/classifier/representation/lifetime, effects, Result
representation, and versioned target adapters independently of private
headers, observation tags, failure-path zero carriers, compiler names, LLVM
types/symbols, debug shadows, and physical placement.

### TOPAL-COMPILER-LIST-FUNCTION-001 — Exact private finite List Function environments

Within one compilation unit, an exact finite `List Function` MAY be
constructed from `Entry` and `Empty`, bound, displayed, passed through an
ordinary private parameter or result, used as a package field, recursively
contained in an admitted Tuple or labeled Record, and decomposed by a complete
List decision. The checked frontend SHALL retain the exact length and one
source-ordered named, symbolic, anonymous, or admitted nested callable identity
for every entry. An opaque, branch-selected, transformed, or otherwise
inexact entry sequence or callable identity SHALL reject before LLVM lowering.

The source List SHALL retain a Topal-owned singly linked representation. Each
Function entry node SHALL contain the existing private i32 Function observation
and an aligned next pointer, and `Empty` SHALL remain null. Capture snapshots
SHALL NOT be embedded in the source nodes. A complete List decision SHALL
attach the first entry's callable facts to the first binding and the remaining
ordered facts to the rest binding. Eventual application SHALL remain one direct
specialized private `fastcc` call; neither node traversal nor Function
observation SHALL dispatch it. Display and DWARF/GDB SHALL expose the source
`List Function` and Function observations without compiler facts.

Each Function entry SHALL add its canonical zero-based source entry index to
the path of `TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`, composed with
enclosing Tuple and Record edges. Private parameters SHALL pass the source List
pointer followed by captures in entry order and existing per-callable capture
order. Private results SHALL return the unchanged pointer followed by those
captures for one-time caller extraction and remapping. An exact nested Function
MAY therefore escape through an entry under
`TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`. Distinct entries and factory
invocations SHALL retain independent snapshots. Function- or
Generator-containing capture state SHALL remain unsupported.

Tests SHALL cover empty, named, symbolic, anonymous, and nested entries;
multiple entry positions and independent factory snapshots; complete decision
application; private parameter/result, package, Tuple, and Record passage;
exact display; interpreter parity and reversible history; direct LLVM IR;
artifact-free opaque/capture/repeated-identity rejection; freestanding
ELF/DWARF; full O0 GDB List/callable values and active frames; the shared
corpus; and separate interpreter/compiler resource baselines.

This rule SHALL add no closure/environment object, environment pointer,
function pointer, indirect call, callback, dispatch table, caller-frame lookup,
foreign dependency, C/C++ runtime, other-language standard library, public
callable/List ABI, or native-ABI revision. Allocation SHALL remain the existing
process-lifetime List nodes. General List Function equality or repeated
identity, operations that lose exact entry facts, opaque or dynamically
selected entries, persistent storage, publication, recursive element
classifiers, and Function containment in other collection representations
remain deferred. Future compiled-library metadata SHALL encode canonical
source-session/scope, exact length and entry order, callable/declaration and
aggregate paths, ordered capture identity/classifier/representation/lifetime,
effects, List representation and ownership, and versioned target adapters
independently of node offsets, private observation tags, compiler names, LLVM
types/symbols, debug shadows, and physical placement.

### TOPAL-COMPILER-ARRAY-FUNCTION-001 — Exact private fixed-size Array Function environments

Within one compilation unit, an exact finite `List Function` MAY be collected
as `Array N Function`, where `N` is its retained exact entry count. The Array
MAY be bound, displayed, queried for count or emptiness, passed through an
ordinary private parameter or result, used as a package field, recursively
contained in an admitted Tuple or labeled Record, and accessed by
`array-at?` with an exact nonnegative index. The checked frontend SHALL retain
the exact extent and one source-ordered named, symbolic, anonymous, or admitted
nested callable identity for every entry. An opaque, branch-selected,
transformed, or otherwise inexact entry sequence or identity SHALL reject
before LLVM lowering.

The source Array SHALL retain the existing Topal-owned 16-byte sequence header:
an i64 entry count followed by a pointer to the immutable source List nodes.
Collection SHALL NOT copy or enlarge those nodes, and capture snapshots SHALL
NOT be embedded in the Array header or entries. An in-bounds exact checked
access SHALL construct the ordinary `Some Function` representation and attach
the selected callable facts to its payload binding. An out-of-bounds exact
index SHALL construct `None Function`. Eventual application SHALL remain one
direct specialized private `fastcc` call; neither Array traversal nor the
Function observation SHALL dispatch it. Display and DWARF/GDB SHALL expose the
source Array and Function observations without compiler facts.

Each Function entry SHALL add its canonical zero-based Array index to the path
of `TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`, composed with enclosing
Tuple and Record edges. Private parameters SHALL pass the source Array pointer
followed by captures in entry order and existing per-callable capture order.
Private results SHALL return the unchanged pointer followed by those captures
for one-time caller extraction and remapping. An exact nested Function MAY
therefore escape through an Array entry under
`TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`. Distinct entries and factory
invocations SHALL retain independent snapshots. Function- or
Generator-containing capture state SHALL remain unsupported.

Tests SHALL cover zero and nonzero extents; named, symbolic, anonymous, and
nested entries; multiple positions and independent factory snapshots;
in-bounds and out-of-bounds checked access with complete Optional decisions;
count, emptiness, private parameter/result, package, Tuple, and Record passage;
exact display; interpreter parity and reversible history; direct LLVM IR;
artifact-free opaque/capture/repeated-identity rejection; freestanding
ELF/DWARF; full O0 GDB Array/callable values and active frames; the shared
corpus; and separate interpreter/compiler resource baselines.

This rule SHALL add no closure/environment object, environment pointer,
function pointer, indirect call, callback, dispatch table, caller-frame lookup,
foreign dependency, C/C++ runtime, other-language standard library, public
callable/Array ABI, or native-ABI revision. Allocation SHALL remain the existing
process-lifetime List nodes, one Array header, and the ordinary in-bounds
Optional Function observation box. General Array Function equality or repeated
identity, dynamic indexing, operations that lose exact entry facts, opaque or
dynamically selected entries, persistent storage, publication, recursive
element classifiers, and Function containment in other collection
representations remain deferred. Future compiled-library metadata SHALL encode
canonical source-session/scope, exact extent and entry order, source collection
relationship, callable/declaration and aggregate paths, ordered capture
identity/classifier/representation/lifetime, effects, Array/List representation
and ownership, and versioned target adapters independently of headers, node
offsets, private observation tags, compiler names, LLVM types/symbols, debug
shadows, and physical placement.

### TOPAL-COMPILER-MAP-FUNCTION-001 — Exact private String-keyed Map Function environments

Within one compilation unit, an exact nonempty `List (String, Function)` MAY be
collected as `Map (String, Function)` under `reject`, `keep-first`, or
`keep-last`. The Map MAY be bound, displayed, queried for count or emptiness,
passed through an ordinary private parameter or result, used as a package
field, recursively contained in an admitted Tuple or labeled Record, and
queried by `map-lookup` with an exact String key. The checked frontend SHALL
retain every exact source key and named, symbolic, anonymous, or admitted
nested callable identity, resolve the collision policy, and retain one value
fact subtree per surviving key in first-key occurrence order. An empty source,
opaque or computed key, branch-selected or otherwise inexact Map/callable, or
lookup without an exact key SHALL reject before LLVM lowering.

The source Map SHALL retain the existing Topal-owned 16-byte sequence header:
an i64 entry count followed by a pointer to 24-byte linked nodes. Each node
SHALL retain the existing String key pointer, the private i32 Function
observation in the value slot, and the next pointer. Collection SHALL NOT embed
captures in the header or nodes. `reject` SHALL diagnose a duplicate exact key;
`keep-first` SHALL retain the first callable and capture facts; `keep-last`
SHALL retain the last callable and capture facts while preserving the first
key's node position. A present exact-key lookup SHALL construct the ordinary
`Some Function` representation and attach only the retained facts for that key;
a missing exact key SHALL construct `None Function`. Eventual application SHALL
remain one direct specialized private `fastcc` call; neither Map traversal nor
the Function observation SHALL dispatch it. Display and DWARF/GDB SHALL expose
the source Map, String keys, and Function observations without compiler facts.

Each surviving Function value SHALL add its exact String key as a semantic
Map-value edge to the path of
`TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`, composed with enclosing Tuple
and Record edges. Private parameters SHALL pass the source Map pointer followed
by captures in resolved entry and existing per-callable capture order. Private
results SHALL return the unchanged pointer followed by those captures for
one-time caller extraction and remapping. An exact nested Function MAY therefore
escape through a Map value under
`TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`. Distinct keys and factory
invocations SHALL retain independent snapshots. Function- or
Generator-containing capture state SHALL remain unsupported.

Tests SHALL cover nonempty named, symbolic, anonymous, and nested Function
values; all three collision policies; present and missing exact-key lookup with
complete Optional decisions; count, emptiness, private parameter/result,
package, Tuple, and Record passage; independent factory snapshots; exact
display; interpreter parity and reversible history; direct LLVM IR;
artifact-free empty/opaque-key/opaque-map/capture/repeated-identity rejection;
freestanding ELF/DWARF; full O0 GDB Map/callable values and active frames; the
shared corpus; and separate interpreter/compiler resource baselines.

This rule SHALL add no closure/environment object, environment pointer,
function pointer, indirect call, callback, dispatch table, caller-frame lookup,
foreign dependency, C/C++ runtime, other-language standard library, public
callable/Map ABI, or native-ABI revision. Allocation SHALL remain the existing
process-lifetime source List nodes, Map nodes and header, and the ordinary
present Optional Function observation box. Empty Map Function collection
remains deferred until shared typed-empty Map semantics are implemented.
General Map Function equality or repeated identity, dynamic lookup, operations
that lose exact key/value facts, Function keys, other key classifiers, opaque
or dynamically selected values, persistent storage, publication, recursive
contained classifiers, and Function containment in other unordered collection
representations remain deferred. Future compiled-library metadata SHALL encode
canonical source-session/scope, collision policy, exact key set and resolved
order, source collection relationship, callable/declaration and semantic
key-based aggregate paths, ordered capture
identity/classifier/representation/lifetime, effects, Map/List representation
and ownership, and versioned target adapters independently of headers, node
offsets, private observation tags, compiler names, LLVM types/symbols, debug
shadows, and physical placement.

### TOPAL-COMPILER-ANONYMOUS-PRODUCT-001 — Private anonymous product patterns

An inferred anonymous function MAY contain a flat positional product parameter
pattern in any source parameter position. Each such pattern SHALL consume one
Tuple operand, require the same field count, and bind its fields in source order
under `TOPAL-FUNCTION-ANONYMOUS-001`. A bound or directly applied function MAY
capture the private immutable data admitted by
`TOPAL-COMPILER-ANONYMOUS-CAPTURE-001`, and a non-capturing function with such a
pattern MAY pass through the admitted private Function-parameter or result
paths.

The complete application operand SHALL be evaluated exactly once before any
field binding. This applies both to an opaque Tuple produced by a call or local
binding and to the outer positional product supplied to a multi-parameter
anonymous function. The checked compiler SHALL preserve any sound field facts
from a direct Tuple construction without re-evaluating its field expressions.
Non-Tuple operands, field-count mismatches, repeated field bindings outside
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`, and field representations
outside the admitted private function boundary SHALL fail before the anonymous
body or LLVM lowering.

The specialized private `fastcc` signature SHALL flatten ordinary binding
parameters and product-pattern fields in lexical source order, followed by any
hidden capture parameters. This machine flattening SHALL NOT change the source
arity or `<anonymous fn/N>` identity. LLVM SHALL own physical AMD64 placement.
DWARF/GDB SHALL expose every non-discarded destructured field as a source-named
parameter with its exact classifier and value, plus any material capture.

The once-only operand binding is compiler-private SSA state, not source-visible
storage. Recursive product patterns are governed by
`TOPAL-COMPILER-ANONYMOUS-NESTED-PATTERN-001`. This rule SHALL introduce no
product-pattern object, environment object, function pointer, indirect call,
closure or Function runtime, foreign dependency, C/C++ runtime, other-language
standard library, public aggregate or callable ABI, or native-ABI revision.
Repeated-name aggregate identity patterns and capturing Function boundaries
outside
`TOPAL-COMPILER-FUNCTION-CAPTURE-PARAMETER-001` and
`TOPAL-COMPILER-FUNCTION-CAPTURE-RESULT-001`, other escape, aggregate
containment outside `TOPAL-COMPILER-FUNCTION-AGGREGATE-001`, publication, and
library metadata/adapters remain deferred.

### TOPAL-COMPILER-ANONYMOUS-NESTED-PATTERN-001 — Recursive anonymous product patterns

Each field of an inferred anonymous Function's positional product pattern MAY
recursively be another positional product pattern. Every product node SHALL
consume one exact Tuple with the same field count, and every leaf binding or
discard SHALL be visited in lexical depth-first, left-to-right order. A
non-Tuple at any product node or a field-count mismatch at any depth SHALL be
rejected before entering the body or lowering LLVM.

The complete application operand SHALL execute exactly once. The checked
frontend SHALL retain it in compiler-private storage when projection is needed
and SHALL derive every nested field only through recursive Tuple projection;
neither an outer field nor a nested initializer may be replayed. The source
Function arity and anonymous display identity SHALL count only top-level
parameter patterns, while the exact private `fastcc` signature SHALL contain
the flattened admitted leaves in lexical order followed by captures. Existing
repeated-name identity and discard semantics SHALL apply across all recursive
leaves. Capturing and returned anonymous Functions MAY use this pattern when
their existing exact private boundary remains admitted.

DWARF/GDB SHALL expose each non-discarded leaf as one source-named parameter
with its exact classifier and value, but SHALL NOT expose compiler-private
outer or nested projection storage. Tests SHALL cover an opaque nested Tuple
result, both nesting directions, mixed top-level parameters, an immutable
capture, a capturing Function result, repeated names and discard, non-Tuple and
nested-arity rejection, exactly-once IR, interpreter parity and reversible
history, freestanding execution, and full O0 GDB frames. This rule SHALL add no
pattern object, allocation, environment object, function pointer, indirect
call, closure or Function runtime, foreign dependency, C/C++ runtime,
other-language standard library, public aggregate or callable ABI, or
native-ABI revision. Named-function header patterns, aggregate containment
outside `TOPAL-COMPILER-FUNCTION-AGGREGATE-001`, dynamic escape/selection,
publication, and library metadata/adapters remain deferred.

### TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001 — Exact repeated anonymous pattern names

A non-discard name MAY recur across the scalar parameters and positional
product leaves of an inferred anonymous function, including recursive leaves
under `TOPAL-COMPILER-ANONYMOUS-NESTED-PATTERN-001`. The first occurrence SHALL
create the sole source binding. Each later occurrence SHALL consume its normal
source field and private machine operand but SHALL require the same exact
classifier and value under `TOPAL-TYPE-MATCH-001`; it SHALL NOT create or
replace a binding. Each `_` occurrence SHALL remain an independent discard.

This increment SHALL admit exact identity for `Unit`, `Completed`, `Effect`,
`Type`, `Function`, `Boolean`, `Int`, `Nat`, a single exact modular classifier,
`Rational`, `Comparison`, `ErrorCode`, a single exact Enum classifier,
`Character`, and `String`. Exact identity SHALL use no evidence forgetting,
implicit conversion, user-visible Equality overload, canonical equivalence, or
approximation. Different classifiers SHALL be rejected before LLVM lowering.

The complete call operand SHALL be evaluated once before projection. Every
repeated-name guard SHALL run in lexical order before the anonymous body. A
failed native guard SHALL write `E-ANONYMOUS-PATTERN-IDENTITY` through the
Topal-owned Linux syscall layer and terminate with status 65; the body SHALL NOT
run. The specialized private `fastcc` signature SHALL retain every consumed
machine operand. DWARF/GDB SHALL expose only the first occurrence as the named
source parameter. Integer, Rational, String, Character, enum-like, and Function
tag checks SHALL lower to their existing exact direct comparisons; a Function
tag SHALL remain observational metadata and SHALL NOT become dispatch.

This rule SHALL introduce no pattern object, matching table, function pointer,
indirect call, closure or Function runtime, foreign dependency, C/C++ runtime,
other-language standard library, public aggregate or callable ABI, or
native-ABI revision. Aggregate repeated-name identity is governed by
`TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001`. Ordinary named-function
header repetition, publication, and library
metadata/adapters remain deferred.

### TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001 — Exact repeated aggregate values

A repeated anonymous-pattern name MAY match the same exact Tuple, Record,
Optional, or List classifier when that aggregate has an already-admitted
private function-boundary representation and complete exact structural
equality. Tuple fields and canonical Record fields SHALL compare recursively in
their semantic order. Optional values SHALL compare their tag and, when
present, their admitted payload. Admitted Lists SHALL compare entries in order
and require equal length. No aggregate comparison SHALL use conversion,
evidence forgetting, user-visible Equality selection, canonical equivalence,
approximation, allocation identity, or inactive representation data.

The initial aggregate set SHALL comprise recursively equality-capable Tuple and
Record values over admitted non-Function leaves; `Optional Int`, `Optional
Rational`, `Optional String`, and `Optional (Int, String)`; `List Int`; and
`List List (Int, String)`. Every later occurrence SHALL retain the exact same
private aggregate parameter representation as its first occurrence. Its guard
SHALL reuse the existing direct field comparisons or Topal-owned Optional/List
comparison primitive before the body. DWARF/GDB SHALL expose only the first
occurrence through its target-derived aggregate or pointer representation.
Mismatch reporting and status SHALL remain those of
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`.

This rule SHALL introduce no generic aggregate matcher, pattern table,
allocation, callback, indirect dispatch, foreign dependency, C/C++ runtime,
other-language standard library, public aggregate ABI, or native-ABI revision.
Sum identity is governed by
`TOPAL-COMPILER-ANONYMOUS-REPEATED-SUM-001`. Result, Range, Generator, refined,
authority-bearing, and Function-containing aggregate identity outside
`TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001`; ordinary
named-function header repetition; publication; and library
metadata/adapters remain deferred.

### TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001 — Exact capture-free Function aggregate values

A repeated anonymous-pattern name MAY match the same exact recursively nested
Tuple or Record classifier containing Function leaves when every Function leaf
retains one exact named, symbolic, non-capturing anonymous, or other
capture-free callable identity under `TOPAL-COMPILER-FUNCTION-AGGREGATE-001`.
Every non-Function leaf SHALL have exact compiler equality already admitted by
`TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001`. A missing or opaque callable
fact SHALL be rejected before LLVM lowering. Capture-bearing leaves are governed
by `TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-AGGREGATE-001`.

The first occurrence SHALL remain the sole source binding and DWARF parameter.
Every later occurrence SHALL retain the same recursive private aggregate
representation and compare fields in semantic order before body entry. A
Function leaf SHALL compare its deterministic private i32 observation field;
that field SHALL NOT select or dispatch executable code. Mismatch reporting
and status SHALL remain those of
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`. LLVM SHALL own physical AMD64
aggregate argument placement from the qualified target data layout.

Tests SHALL cover named, symbolic, and non-capturing anonymous Function leaves,
nested Tuple/Record values, exact success and mismatch behavior, structural
guarded IR, interpreter parity and reversible history, freestanding execution,
and full O0 GDB values/frames. This
rule SHALL introduce no closure or environment object, allocation, generic
matcher, pattern table, function pointer, indirect call, callback, dispatch
table, foreign dependency, C/C++ runtime, other-language standard library,
public aggregate/callable ABI, or native-ABI revision. Dynamic aggregate
selection, Function containment outside admitted Tuple/Record/Optional/Sum/Result/List/Array/Map paths,
ordinary named-function header repetition, publication, and library adapters
remain deferred. Future
compiled-library metadata SHALL encode canonical aggregate paths and stable
callable/representation identities independently of private LLVM types,
observation tags, and target-specific argument placement.

### TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-001 — Exact captured anonymous Function values

A repeated anonymous-pattern name MAY match two values produced by the same
anonymous Function source when both values retain the same ordered capture
schema and every represented capture classifier has exact compiler equality.
Exact Function identity SHALL comprise the anonymous source identity followed
by the already-evaluated captured values in their retained order. The source
identity SHALL be canonical across private specializations of the same
anonymous expression; a compiler-private observation tag MAY represent it but
the numeric tag SHALL NOT define a public or serialized identity. A capture
with missing facts, a different schema, or no admitted exact equality SHALL be
rejected before LLVM lowering.

The first Function occurrence SHALL remain the sole source binding and DWARF
parameter. Every later occurrence SHALL retain its ordinary observation field
and its captures as deterministic hidden private operands. Guards SHALL compare
the source identity first and then each capture in order before body entry,
using existing exact direct comparisons. A mismatch SHALL use
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`; no observation tag SHALL
dispatch executable code. LLVM SHALL own physical AMD64 placement for the exact
private prototype.

Tests SHALL cover equal and unequal captures from one factory source,
canonical source identity across specializations, rejected non-equality capture
state, guard order, interpreter parity and reversible history, freestanding
execution, and full O0 GDB values/frames. This rule SHALL introduce no closure
or environment object, allocation, pattern table, function pointer, indirect
call, dispatch table, foreign dependency, C/C++ runtime, other-language
standard library, public callable ABI, or native-ABI revision. Captured named
identity is governed by
`TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-NAMED-FUNCTION-001`. Unsupported
capture classifiers, publication, and library adapters remain deferred. Future
compiled-library metadata SHALL encode the stable callable source identity and
ordered capture schema, classifiers, semantic equality requirements,
representation identity, lifetime/effects, and target adapter without
serializing private observation tags, parameter names, LLVM types, or physical
argument placement.

### TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-NAMED-FUNCTION-001 — Exact captured named Function values

A repeated anonymous-pattern name MAY match two captured named Function values
when both retain the same stable declaration identity, the same ordered capture
schema, and exact compiler equality for every represented capture classifier.
The initial admitted set SHALL be a non-escaping nested lexical Function used
within its defining invocation. Exact named identity SHALL comprise its source
name and declaration identity independently of a compiler-private observation
tag or private specialization. When declaration identities differ, the
observation-field guard makes the values unequal and capture comparison is not
required. Missing declaration facts, inconsistent required capture schemas, or
a required capture without admitted exact equality SHALL be rejected before
LLVM lowering.

The first Function occurrence SHALL remain the sole source binding and DWARF
parameter. When identities match, both already-evaluated capture snapshots
SHALL follow the source operands as deterministic hidden private operands.
Guards SHALL compare the callable identity first and then each required capture
in retained order before body entry. A mismatch SHALL use
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`; no observation tag SHALL
dispatch executable code. LLVM SHALL own physical AMD64 placement for the exact
private prototype.

Tests SHALL cover a captured nested Function repeated and then invoked,
different captured nested declarations whose identity guard mismatches without
requiring capture equality, same-declaration non-equality capture rejection,
direct ordered IR, interpreter parity and reversible history, freestanding
execution, and full O0 GDB values/frames. This rule SHALL introduce no closure
or environment object, allocation, pattern table, function pointer, indirect
call, dispatch table, foreign dependency, C/C++ runtime, other-language
standard library, public callable ABI, or native-ABI revision. Escaping nested
Functions, unsupported capture classifiers, ordinary named-function header
repetition, publication, and library adapters remain deferred. Future
compiled-library metadata SHALL encode stable declaration/source identity,
ordered capture schemas and classifiers, semantic equality requirements,
representation identity, lifetime/effects, and target adapters without
serializing source offsets, private observation tags, hidden operand names or
layout, LLVM types, or physical argument placement.

### TOPAL-COMPILER-ANONYMOUS-REPEATED-CAPTURED-FUNCTION-AGGREGATE-001 — Exact captured Function aggregate values

A repeated anonymous-pattern name MAY match the same exact recursively nested
Tuple or Record classifier containing captured Function leaves when both
operands retain complete callable facts at every Function path. Each ordinary
field SHALL compare in the semantic order established by
`TOPAL-COMPILER-ANONYMOUS-REPEATED-FUNCTION-AGGREGATE-001`. When every pair of
corresponding Function leaves has the same named, symbolic, nested, or canonical
anonymous source identity, the two values SHALL additionally retain the same
path-ordered capture schema and every represented capture classifier SHALL have
exact compiler equality. If any callable identity differs, its observation
field makes the aggregate unequal and no capture comparison is required.
Missing callable facts, inconsistent capture schemas, or a required capture
without admitted exact equality SHALL be rejected before LLVM lowering.

The first aggregate occurrence SHALL remain the sole source binding and DWARF
parameter. Its captures and, when required, the later occurrence's captures
SHALL be deterministic hidden private operands ordered by canonical aggregate
path and retained capture order. Guards SHALL compare ordinary aggregate fields
first and then required capture pairs in that same deterministic order before
body entry. A mismatch SHALL use
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`; observation tags SHALL remain
non-dispatching, and LLVM SHALL own physical AMD64 placement for the exact
private prototype.

Tests SHALL cover captured anonymous leaves in Record and nested Tuple/Record
values, captured nested named leaves, equal and unequal capture payloads,
callable-identity mismatch without unnecessary capture equality, rejected
non-equality capture state when identities match, checked path/capture metadata,
direct guard order, interpreter parity and reversible history, freestanding
execution, and full O0 GDB values/frames. This rule SHALL introduce no closure
or environment object, allocation, pattern table, function pointer, indirect call, dispatch
table, foreign dependency, C/C++ runtime, other-language standard library,
public aggregate/callable ABI, or native-ABI revision. Dynamic aggregate
selection, Function containment outside admitted Tuple/Record/Optional/Sum/Result/List/Array/Map paths,
unsupported capture classifiers, ordinary named-function header repetition,
publication, and library adapters remain deferred. Future compiled-library
metadata SHALL encode canonical aggregate paths, stable callable identities, ordered capture schemas
and classifiers, semantic equality requirements, representation identity,
lifetime/effects, and target adapters without serializing private observation
tags, hidden operand names or layout, LLVM types, or physical argument
placement.

### TOPAL-COMPILER-ANONYMOUS-REPEATED-SUM-001 — Exact nominal Sum repeated identity

A repeated anonymous-pattern name MAY match the same exact nominal `Union` or
positional `Variant` classifier when every possible payload has exact compiler
identity already admitted by
`TOPAL-COMPILER-ANONYMOUS-REPEATED-AGGREGATE-001` or recursively by this rule.
Tuple and Record fields MAY recursively contain such a Sum. The two operands
SHALL have the same nominal classifier; structural similarity between distinct
Sum declarations SHALL NOT suffice. Function, Range, Result, refined,
authority-bearing, or any other payload without admitted exact identity SHALL
be rejected before LLVM lowering or artifact publication.

The guard SHALL first compare the represented alternative tags. Different tags
SHALL mismatch without observing either payload. Equal tags SHALL select that
one active alternative and compare only its payload recursively; inactive
private representation fields SHALL NOT affect source identity or be observed.
Payload-free alternatives SHALL match from the equal tag alone. This path SHALL
run before the anonymous body and SHALL reuse the diagnostic and status of
`TOPAL-COMPILER-ANONYMOUS-REPEATED-PATTERN-001`. It SHALL NOT by itself publish
the general source `Equality` capability for Sum values.

On Linux x86-64 the source operands SHALL retain the existing exact private
tag-plus-payload LLVM aggregate and matching `fastcc` prototypes. LLVM
`switch`, direct recursive comparisons, and an `i1` `phi` MAY lower the
active-payload selection, while LLVM owns physical register, stack, branch, and
aggregate coercion for the qualified target. The first occurrence SHALL remain
the sole source binding and DWARF/GDB parameter; its semantic nominal Sum type
and active value SHALL remain inspectable.

Tests SHALL cover labeled Union and positional Variant values; payload-free,
Int, String, Tuple, and recursively nested Sum payloads; equal values; same-tag
payload mismatch; distinct-tag short-circuit; unsupported payload rejection
before publication; direct guarded IR;
interpreter parity and reversible history; freestanding execution; and full O0
GDB values/frames. This rule SHALL introduce no generic matcher, Sum-equality
runtime, pattern table, allocation, callback, indirect dispatch, foreign
dependency, C/C++ runtime, other-language standard library, public aggregate
ABI, or native-ABI revision. General derived Sum Equality is governed by
`TOPAL-COMPILER-SUM-EQUALITY-001`; recursive Sum declarations,
Function-containing Sums outside the exact selected-value rules of
`TOPAL-COMPILER-SUM-FUNCTION-001`, persistent storage, publication, and library
adapters remain deferred. Future compiled-library metadata SHALL
encode the stable nominal identity, positional/labeled form, ordered
alternative identities, payload schemas, semantic identity/equality
requirements, representation identity, and target adapters independently of
private numeric tags, inactive storage, LLVM types, and physical placement.

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

Exact application through a retained local alias and exact defining-context or
live-root environment parameters are governed by
`TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001`. Those environment parameters
SHALL remain distinct from the lexical captures permitted here.

The nested function name SHALL remain non-escaping compiler metadata. Exact
same-scope retention and application through a local alias are governed by
`TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001`; any parameter or aggregate
boundary is governed only by its later explicit Function-boundary rule. Other
return, storage, boundary, or escape forms SHALL be rejected. The lowering
SHALL require no caller-frame reference, environment object, closure allocation
or runtime, function pointer, indirect call, foreign dependency, C/C++ runtime,
other-language standard library, public callable ABI, or native ABI revision.

Declarations inside nested lexical/decision blocks and published, static,
measured, constrained, or effectful nested functions; nested overload sets,
recursion, sibling calls, collisions with visible or active named callables,
anonymous captures, Scope, Function, Constraint, or refined-evidence captures;
context/root environments beyond
`TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001`; escaping closures; and
public/library closure metadata remain outside this increment and SHALL be
rejected rather than receiving a provisional closure representation.

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

Label association beyond a declaration-order prefix is governed by
`TOPAL-COMPILER-PACKAGED-ASSOCIATION-ORDER-001`. Two packaged operands and
packages mixed with an unpackaged operand are governed by
`TOPAL-COMPILER-COMPOUND-PACKAGED-OPERAND-001`. Exact capture-free Tuple and
Record fields are governed by
`TOPAL-COMPILER-STRUCTURED-PACKAGED-FIELD-001`; exact nominal Sum fields are
governed by `TOPAL-COMPILER-SUM-PACKAGED-FIELD-001`; exact direct Function
fields are governed by `TOPAL-COMPILER-FUNCTION-PACKAGED-FIELD-001`; exact
Function-containing Tuple and Record fields are governed by
`TOPAL-COMPILER-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`. Exact represented
List, Optional, Result, and Range fields are governed by
`TOPAL-COMPILER-CONTAINER-PACKAGED-FIELD-001`. Exact Array, Set, Bag, and Map
fields are governed by `TOPAL-COMPILER-COLLECTION-PACKAGED-FIELD-001`. Exact
root and root-alias Scope fields are governed by
`TOPAL-COMPILER-SCOPE-PACKAGED-FIELD-001`. Other
non-scalar fields,
nested package declarations, opaque package values, and defaults that depend on
invocation or captured bindings remain outside this increment and SHALL be
rejected rather than changing evaluation order or choosing a public memory ABI.

### TOPAL-COMPILER-PACKAGED-ASSOCIATION-ORDER-001 — Label-based scalar package association

The compiler SHALL extend the one admitted scalar operand package of
`TOPAL-COMPILER-PACKAGED-OPERAND-001` so a labeled product may supply each
unique known field in any source order. Every nondefaulted field SHALL be
present, and any omitted field MAY occur at any declaration position only when
it has a checked closed default. Unknown or duplicate labels, missing required
fields, and classifier mismatches SHALL reject the call before LLVM lowering or
artifact publication.

Every supplied expression SHALL execute exactly once in labeled source order.
When declaration order differs, the checked model SHALL introduce
compiler-private immutable bindings in source order, then adapt and permute
references to those values into declaration field order. Omitted closed
defaults SHALL execute once in declaration order after all supplied
expressions. These private bindings SHALL NOT become source variables or DWARF
locals.

The callee SHALL retain the existing exact flattened private `fastcc` signature
and source field DWARF parameters in declaration order. LLVM SHALL own AMD64
register and stack placement. This extension SHALL introduce no package
aggregate at the call boundary, `byval`, `sret`, `inalloca`, `preallocated`,
package runtime, allocation, foreign dependency, C/C++ runtime, other-language
standard library, public aggregate ABI, or native-ABI revision. Future
compiled-library metadata SHALL encode stable field identities, declaration
order, default semantics and dependencies, evaluation effects, representation
identity, and target adapters independently of compiler-private binding names,
LLVM types, and physical placement. Compound packages are governed by
`TOPAL-COMPILER-COMPOUND-PACKAGED-OPERAND-001`. Exact capture-free Tuple and
Record fields are governed by
`TOPAL-COMPILER-STRUCTURED-PACKAGED-FIELD-001`; exact nominal Sum fields are
governed by `TOPAL-COMPILER-SUM-PACKAGED-FIELD-001`; exact direct Function
fields are governed by `TOPAL-COMPILER-FUNCTION-PACKAGED-FIELD-001`; exact
Function-containing Tuple and Record fields are governed by
`TOPAL-COMPILER-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`. Exact represented
List, Optional, Result, and Range fields are governed by
`TOPAL-COMPILER-CONTAINER-PACKAGED-FIELD-001`. Exact Array, Set, Bag, and Map
fields are governed by `TOPAL-COMPILER-COLLECTION-PACKAGED-FIELD-001`. Exact
root and root-alias Scope fields are governed by
`TOPAL-COMPILER-SCOPE-PACKAGED-FIELD-001`. Other
non-scalar fields,
nested package declarations, opaque package values, invocation- or
capture-dependent defaults, and public package adapters remain deferred.

### TOPAL-COMPILER-COMPOUND-PACKAGED-OPERAND-001 — Two and mixed scalar packages

The compiler SHALL extend the admitted closed scalar package to either or both
of a function's two syntactic operands. An unpackaged operand mixed with a
package SHALL use an admitted non-callable scalar classifier. Both syntactic
operands remain mandatory. Parameter names SHALL be unique across both packages
and any unpackaged operand; unknown or duplicate labels, missing required
fields, classifier mismatches, and duplicate parameter names SHALL reject the
call before LLVM lowering or artifact publication.

Every explicit operand expression and package-field expression SHALL execute
exactly once in global source order: the left operand before the right operand,
and fields within each product in their source order. The checked model SHALL
retain those values through compiler-private immutable bindings. Omitted closed
defaults SHALL then execute once in syntactic-operand order and field declaration
order. The direct call SHALL receive unpackaged operands and package fields in
syntactic-operand order, with fields in declaration order. The private bindings
SHALL NOT become source variables or DWARF locals.

The callee SHALL use one exact flattened private `fastcc` parameter per
unpackaged operand or package field. LLVM SHALL own AMD64 register and stack
placement. DWARF/GDB SHALL expose only the source parameter and field names,
classifiers, values, and ordinary function frame. The lowering SHALL introduce
no package aggregate at the call boundary, `byval`, `sret`, `inalloca`,
`preallocated`, package runtime, allocation, foreign dependency, C/C++ runtime,
other-language standard library, public aggregate ABI, or native-ABI revision.

Future compiled-library metadata SHALL additionally preserve syntactic-operand
partition and order beside stable field identities, field declaration order,
default semantics and dependencies, evaluation effects, representation
identity, and target adapters. It SHALL remain independent of compiler-private
binding names, LLVM types, and physical placement. Exact capture-free Tuple and
Record fields are governed by
`TOPAL-COMPILER-STRUCTURED-PACKAGED-FIELD-001`; exact nominal Sum fields are
governed by `TOPAL-COMPILER-SUM-PACKAGED-FIELD-001`; exact direct Function
fields are governed by `TOPAL-COMPILER-FUNCTION-PACKAGED-FIELD-001`; exact
Function-containing Tuple and Record fields are governed by
`TOPAL-COMPILER-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`. Exact represented
List, Optional, Result, and Range fields are governed by
`TOPAL-COMPILER-CONTAINER-PACKAGED-FIELD-001`. Exact Array, Set, Bag, and Map
fields are governed by `TOPAL-COMPILER-COLLECTION-PACKAGED-FIELD-001`. Exact
root and root-alias Scope fields are governed by
`TOPAL-COMPILER-SCOPE-PACKAGED-FIELD-001`. Other Function-containing
aggregates, other non-scalar fields, nested package
declarations, opaque compound package values, invocation- or capture-dependent
defaults, recursion through compound package signatures, and public package
adapters remain deferred.

### TOPAL-COMPILER-STRUCTURED-PACKAGED-FIELD-001 — Exact Tuple and Record package fields

The compiler SHALL admit an exact Tuple or Record classifier as a field of the
one- or two-operand packages governed by the preceding package rules when the
complete structural classifier is already admitted by the private function ABI
and contains no Function value or capture environment. Structured supplied
values and closed structured defaults SHALL undergo the same exact classifier
adaptation as ordinary private function parameters. Nested Tuple and Record
classifiers MAY compose recursively. Function-containing Tuple and Record
fields are governed by
`TOPAL-COMPILER-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`. An unsupported leaf
classifier, mismatched shape, or non-closed default SHALL reject before LLVM
lowering or artifact publication.

Each structured field SHALL remain one source field and one exact private
`fastcc` parameter; its internal components SHALL NOT become additional package
fields or a package-level aggregate. Explicit structured expressions SHALL be
retained once in the source order established by the package rules, closed
defaults SHALL execute afterward in operand/field declaration order, and the
callee arguments SHALL remain in operand/field declaration order. LLVM SHALL
own the AMD64 register and stack placement of each exact aggregate parameter.

DWARF/GDB SHALL expose the source field name, complete Tuple or Record
classifier, recursively structured value, and ordinary function frame. The
compiler-private ordering bindings SHALL remain absent from source debugging.
The lowering SHALL add no `byval`, `sret`, `inalloca`, `preallocated`, package
runtime, allocation beyond the value's existing private representation, foreign
dependency, C/C++ runtime, other-language standard library, public aggregate
ABI, or native-ABI revision.

Future compiled-library metadata SHALL preserve each field's complete canonical
structural classifier and representation identity beside operand partition,
field identity/order, default semantics/dependencies, evaluation effects, and
target adapters. Exact Function-containing Tuple and Record fields are governed
by `TOPAL-COMPILER-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`. Nested package
declarations, opaque whole-package values, invocation- or capture-dependent
defaults, recursive structured package signatures, public adapters, and other
unsupported non-scalar fields remain deferred. Exact nominal Sum fields are
governed by `TOPAL-COMPILER-SUM-PACKAGED-FIELD-001`; exact direct Function
fields are governed by `TOPAL-COMPILER-FUNCTION-PACKAGED-FIELD-001`.

### TOPAL-COMPILER-SUM-PACKAGED-FIELD-001 — Exact nominal Sum package fields

The compiler SHALL admit an exact nominal Sum classifier as a field of the one-
or two-operand packages governed by the package rules when every declared
alternative payload is already admitted by the private Sum function ABI.
Function-bearing payloads SHALL additionally satisfy
`TOPAL-COMPILER-SUM-FUNCTION-001`. Supplied Sum values and
closed Sum defaults SHALL retain their exact nominal identity, tag, and active
payload through ordinary classifier adaptation. Unsupported payloads, nominal
mismatches, and non-closed defaults SHALL reject before LLVM lowering or
artifact publication.

Each Sum field SHALL remain one source field and one exact private `fastcc`
parameter using its existing tag-plus-payload representation; alternatives and
payload components SHALL NOT become package fields. Explicit Sum expressions
SHALL execute once in package source order, closed defaults SHALL execute
afterward in operand/field declaration order, and callee arguments SHALL remain
in operand/field declaration order. LLVM SHALL own AMD64 physical placement of
the exact Sum parameter.

DWARF/GDB SHALL expose the source field name, nominal Sum classifier, active
alternative/payload value, and ordinary function frame. Compiler-private
ordering bindings SHALL remain absent from source debugging. The lowering SHALL
add no `byval`, `sret`, `inalloca`, `preallocated`, package runtime, allocation
beyond the Sum value's existing private representation, foreign dependency,
C/C++ runtime, other-language standard library, public aggregate ABI, or
native-ABI revision.

Future compiled-library metadata SHALL preserve the Sum's canonical nominal
identity, complete alternatives/payload classifiers, and representation identity
beside operand/field/default/effect semantics and target adapters. Nested
package declarations, opaque whole-package values, Function-containing Sum
payloads outside `TOPAL-COMPILER-SUM-FUNCTION-001`, dependent defaults,
recursive Sum package signatures, public adapters, and other unsupported
non-scalar fields remain deferred.

### TOPAL-COMPILER-FUNCTION-PACKAGED-FIELD-001 — Exact direct Function package fields

The compiler SHALL admit an exact Function value as a direct field of the one-
or two-operand packages governed by the package rules when its callable identity
and any represented immutable capture snapshot are already admitted by the
private Function parameter ABI. Supplied values and closed defaults SHALL retain
the same named, symbolic, anonymous, or non-escaping nested callable facts used
by an ordinary Function parameter. Opaque, dynamically selected, escaping, or
otherwise unrepresentable callable values SHALL reject before LLVM lowering or
artifact publication.

When source-order normalization introduces a compiler-private binding for a
Function field, the binding SHALL retain the exact callable and capture facts
beside the runtime Function observation tag. The initializer SHALL execute once
in package source order. The call SHALL pass one source Function field in
operand/field declaration order followed by any deterministic hidden capture
parameters required by the existing specialized callable boundary. Hidden
captures SHALL NOT become package fields or source parameters.

LLVM definitions and calls SHALL use matching private `fastcc` Function-tag and
capture parameters and SHALL leave physical AMD64 placement to LLVM. Application
in the callee SHALL remain a direct specialized call rather than function-pointer
dispatch. DWARF/GDB SHALL expose the source Function field identity/value and
ordinary frame while compiler-private ordering bindings and hidden capture
transport remain absent from source debugging.

The lowering SHALL add no package aggregate, function pointer, indirect call,
closure allocation/runtime, `byval`, `sret`, `inalloca`, `preallocated`, foreign
dependency, C/C++ runtime, other-language standard library, public callable ABI,
or native-ABI revision. Future compiled-library metadata SHALL preserve callable
source/declaration identity, overload set, capture schema and equality,
representation/lifetime/effect semantics, operand/field/default semantics, and
target adapters. Exact Function-containing Tuple and Record fields are governed
by `TOPAL-COMPILER-FUNCTION-AGGREGATE-PACKAGED-FIELD-001`. Function containment
in other aggregates, dynamic escape/selection, nested package declarations,
opaque whole-package values, dependent defaults, recursive callable package
signatures, public adapters, and other unsupported fields remain deferred.

### TOPAL-COMPILER-FUNCTION-AGGREGATE-PACKAGED-FIELD-001 — Exact Function aggregate package fields

The compiler SHALL admit an exact Tuple or Record containing one or more
Function leaves as a complete field of a one- or two-operand package when the
same complete value is already admitted by the private Function aggregate
parameter ABI of `TOPAL-COMPILER-FUNCTION-AGGREGATE-001` and
`TOPAL-COMPILER-FUNCTION-AGGREGATE-CAPTURE-001`. Every Function leaf SHALL have
one exact named, symbolic, anonymous, or non-escaping nested callable identity
and a fully represented immutable capture snapshot. Missing, opaque,
dynamically selected, escaping, or otherwise unrepresentable callable facts
SHALL reject before LLVM lowering or artifact publication.

When package source-order normalization introduces a compiler-private binding,
the binding SHALL retain the aggregate's complete recursive structural fact
tree, canonical Function-leaf paths, callable identities, and capture facts
beside its runtime aggregate value. The initializer SHALL execute once in
package source order. Closed aggregate defaults SHALL execute afterward in
operand/field declaration order, and the direct call SHALL pass the complete
aggregate in operand/field declaration order without decomposing its members
into package fields.

The source aggregate parameter SHALL retain its existing exact target-derived
LLVM aggregate type. Any captures required by its Function leaves SHALL follow
as deterministic canonical-path-ordered hidden parameters under the existing
private `fastcc` specialization, and application in the callee SHALL remain a
direct call. LLVM SHALL own AMD64 aggregate classification and physical
placement. DWARF/GDB SHALL expose the one source Tuple or Record field with its
complete value and ordinary frame; compiler-private ordering bindings and
hidden capture transport SHALL remain absent from source debugging.

Tests SHALL cover reordered once-only scalar and Function-aggregate-producing
calls, capture-free Record and capture-bearing Tuple fields, a closed symbolic
Function-aggregate default, exact results, opaque whole-package rejection,
recursive checked facts, exact direct IR, interpreter modes, reversible
history, the shared corpus and separate resource baselines, freestanding
ELF/DWARF validation, and O0 GDB aggregate fields/frames.

The lowering SHALL add no package-level aggregate, member decomposition,
function pointer, indirect call, closure allocation/runtime, `byval`, `sret`,
`inalloca`, `preallocated`, foreign dependency, C/C++ runtime, other-language
standard library, public callable/aggregate ABI, or native-ABI revision. Future
compiled-library metadata SHALL preserve each aggregate field's complete
canonical classifier and representation identity, every Function leaf's
canonical path, callable source/declaration identity and overload set, capture
schema/equality/lifetime/effect semantics, operand/field/default semantics, and
target adapters independently of private binding names, LLVM types, tags, and
physical placement. Optional Function fields are governed by
`TOPAL-COMPILER-OPTIONAL-FUNCTION-001`; exact nominal Sum Function fields are
governed by `TOPAL-COMPILER-SUM-FUNCTION-001`; exact arithmetic Result Function
fields are governed by `TOPAL-COMPILER-RESULT-FUNCTION-001`; exact finite List
Function fields are governed by `TOPAL-COMPILER-LIST-FUNCTION-001`; exact
fixed-size Array Function fields are governed by
`TOPAL-COMPILER-ARRAY-FUNCTION-001`; exact nonempty String-keyed Map Function
fields are governed by `TOPAL-COMPILER-MAP-FUNCTION-001`. Function containment in other aggregates;
dynamic escape/selection; dependent defaults; nested package
declarations; recursive callable package signatures; persistent storage;
publication; and public adapters remain deferred.

### TOPAL-COMPILER-CONTAINER-PACKAGED-FIELD-001 — Exact represented container package fields

The compiler SHALL admit exact List, Optional, Result, and Range classifiers as
complete fields of a one- or two-operand package when that exact classifier is
already admitted by the corresponding private function parameter ABI. A List
element classifier, Optional value classifier, Result success classifier and
error domain, or Range endpoint classifier SHALL remain within that existing
represented boundary and SHALL contain no Function value or capture
environment under this rule. Exact `Optional Function` fields are instead
governed by `TOPAL-COMPILER-OPTIONAL-FUNCTION-001`; exact arithmetic
`Result Function` fields are governed by `TOPAL-COMPILER-RESULT-FUNCTION-001`;
exact finite `List Function` fields are governed by
`TOPAL-COMPILER-LIST-FUNCTION-001`; exact fixed-size `Array Function` fields are
governed by `TOPAL-COMPILER-ARRAY-FUNCTION-001`; exact nonempty
`Map (String, Function)` fields are governed by
`TOPAL-COMPILER-MAP-FUNCTION-001`.
Unsupported contained classifiers and mismatched container classifiers SHALL reject before LLVM
lowering or artifact publication.

Each explicit represented-container expression SHALL execute once in package
source order and be retained with its exact checked container classifier. A
closed default whose analyzed value already has the exact declared container
classifier SHALL execute afterward in operand/field declaration order. The
direct call SHALL pass complete values in operand/field declaration order.
Implicit context-dependent construction or lifting of a differently classified
default, including lifting a bare success value into Result, remains deferred
and SHALL reject rather than receive a compiler-only meaning.

Each source field SHALL remain one exact private pointer-carrier `fastcc`
parameter using the container's existing representation; contained values
SHALL NOT become package fields. LLVM SHALL own AMD64 register and stack
placement. Existing process-lifetime List nodes MAY be allocated while the List
value is constructed, but package normalization SHALL add no allocation or
change that lifetime. DWARF/GDB SHALL expose the source field name, complete
classifier and value, and ordinary function frame. Compiler-private ordering
bindings SHALL remain absent from source debugging.

Tests SHALL cover List, Optional, Result, and Range fields; reordered once-only
container-producing calls; one exact closed Optional default; exact results;
unsupported-field artifact rejection; checked classifier retention; matching
direct pointer-carrier IR; all interpreter modes; reversible history; the
shared corpus and separate resource baselines; freestanding ELF/DWARF
validation; and O0 GDB container values/frames.

The lowering SHALL add no package aggregate, contained-value decomposition,
`byval`, `sret`, `inalloca`, `preallocated`, package runtime, allocation beyond
existing container construction, foreign dependency, C/C++ runtime,
other-language standard library, public container ABI, or native-ABI revision.
Future compiled-library metadata SHALL preserve the canonical generic
constructor, contained classifier or Result error domain, Range endpoint
classifier, representation/lifetime/default/effect semantics, stable
operand/field identities and order, and target adapters independently of
compiler-private binding names, LLVM pointer types, and physical placement.
Function-containing containers outside exact `Optional Function`, arithmetic
`Result Function`, finite `List Function`, fixed-size `Array Function`, and
nonempty exact `Map (String, Function)`,
unsupported List elements and container payloads, context-dependent defaults, nested
package declarations, opaque whole-package values, recursive package
signatures, persistent container storage, publication, and public adapters
remain deferred.

### TOPAL-COMPILER-COLLECTION-PACKAGED-FIELD-001 — Exact represented collection package fields

The compiler SHALL recognize exact `Array (N, Int)`, `Set Int`, `Bag Int`, and
`Map (String, Int)` classifiers in private function parameter and result
headers, and SHALL admit values with those classifiers as complete fields of a
one- or two-operand package. `N` SHALL be a statically parsed nonnegative
extent. Other element, key, or value classifiers and mismatched collection
classifiers SHALL reject before LLVM lowering or artifact publication rather
than being accepted because they share a pointer carrier. Exact
`Array (N, Function)` fields are instead governed by
`TOPAL-COMPILER-ARRAY-FUNCTION-001`; exact nonempty
`Map (String, Function)` fields are governed by
`TOPAL-COMPILER-MAP-FUNCTION-001`.

Each explicit collection-producing expression SHALL execute once in package
source order and retain its exact constructor, extent where applicable, and
element/key/value classifiers in the checked model. The direct call SHALL pass
the complete values in operand/field declaration order. Closed defaults remain
subject to the existing exact checked-default rule; a collection construction
that the frontend cannot already analyze as the declared classifier SHALL
reject rather than receive a compiler-only default meaning.

Each source field SHALL remain one exact private pointer-carrier `fastcc`
parameter using the collection's existing representation. Entries, keys,
values, multiplicities, and collision-resolution state SHALL NOT become
package fields. LLVM SHALL own AMD64 register and stack placement. Existing
process-lifetime allocations MAY occur while Array, Set, Bag, or Map values are
constructed, but package normalization SHALL add no allocation or change their
lifetime. DWARF/GDB SHALL expose the source field name, complete classifier and
value, and ordinary function frame. Compiler-private ordering bindings SHALL
remain absent from source debugging.

Tests SHALL cover all four exact collection families; reordered once-only
collection-producing calls; positional parity; exact results; unsupported or
mismatched classifier rejection before artifact publication; checked
classifier retention; matching direct pointer-carrier IR; all interpreter
modes; reversible history; the shared corpus and separate resource baselines;
freestanding ELF/DWARF validation; and O0 GDB collection values and frames.

The lowering SHALL add no package aggregate, entry decomposition, `byval`,
`sret`, `inalloca`, `preallocated`, package runtime, allocation beyond existing
collection construction, foreign dependency, C/C++ runtime, other-language
standard library, public collection ABI, or native-ABI revision. Future
compiled-library metadata SHALL preserve the canonical constructor; Array
extent; element or key/value classifiers; ordering, uniqueness, multiplicity,
and collision semantics; representation/lifetime/default/effect semantics;
stable operand/field identities and order; and target adapters independently
of compiler-private binding names, LLVM pointer types, and physical placement.
Function-containing collections outside exact `Array (N, Function)` and
nonempty exact `Map (String, Function)`, other generic specializations,
context-dependent or otherwise unanalyzable defaults, nested package
declarations, opaque whole-package values, recursive package signatures,
persistent collection storage, publication, and public adapters remain
deferred.

### TOPAL-COMPILER-SCOPE-PACKAGED-FIELD-001 — Exact root Scope package fields

The compiler SHALL admit the live source-root namespace and an exact retained
root alias as a complete `Scope` field of a one- or two-operand package governed
by the preceding package rules. A supplied alias SHALL retain the namespace
identity, declarations visible at its binding, function overload order, and
represented data facts required by
`TOPAL-COMPILER-NAMESPACE-BOUNDARY-001`. A closed default of `root` SHALL be
admitted. An opaque or computed Scope, a nested or non-root namespace, and a
live `root` argument formed inside a compiled function SHALL reject before LLVM
lowering or artifact publication.

Each explicit field initializer SHALL execute once in package source order. A
compiler-private ordering binding for a Scope field SHALL retain the complete
namespace facts beside the sealed observation value, and the direct call SHALL
receive source fields in operand/field declaration order. The callee SHALL use
the existing specialized Scope boundary: one explicit compiler-private `i32`
Scope value followed by each represented immutable namespace data value in its
deterministic existing order. Qualified function selection SHALL remain static
and direct. LLVM SHALL own AMD64 register and stack placement, and package
normalization SHALL add no allocation or change namespace lifetime or identity.

DWARF/GDB SHALL expose the source Scope field, its `Scope` classifier and
namespace observation, each material represented data argument, and the
ordinary function frame. Compiler-private source-order bindings SHALL remain
absent from source debugging. Tests SHALL cover reordered once-only
Scope/scalar fields, positional parity, a closed `root` default, exact result,
function-body live-root artifact rejection, checked namespace-fact retention,
matching direct IR, all interpreter modes, reversible history, the shared
corpus and separate resource baselines, freestanding ELF/DWARF validation, and
O0 GDB Scope/data values and frames.

The lowering SHALL add no package aggregate, namespace table, dynamic lookup,
function pointer, indirect call, environment or package allocation, `byval`,
`sret`, `inalloca`, `preallocated`, foreign dependency, C/C++ runtime,
other-language standard library, public Scope ABI, or native-ABI revision.
Future compiled-library metadata SHALL preserve namespace identity and snapshot
position; member visibility and declaration order; complete function overload,
generator, and represented-data member schemas; hidden data classifier/order,
lifetime, and effect facts; package operand/field identity and order; default
semantics and dependencies; and versioned target adapters independently of the
private `i32` tag, hidden parameter names, LLVM types, symbols, and physical
placement. Opaque, computed, nested, non-root, external, escaping, result, and
public/library Scope environments remain deferred.

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
Scope, or Function members, anonymous captures, escaping functions, qualified
root access not admitted by `TOPAL-COMPILER-FUNCTION-ROOT-DATA-001`, and
public/library context environments remain outside this increment and SHALL be
rejected. Exact scalar forwarding between compiled functions is governed by
`TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001`.

### TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001 — Private defining-context forwarding

For a call from the source entry frame through a finite acyclic chain of
statically named ordinary root functions, the checked frontend SHALL compute
the transitive exact `@ member` selections before instantiating the outer
function. Each selected member SHALL come from the immutable context snapshot
of the function containing that direct selection, so a member declared after
that selecting function SHALL remain unavailable. Each intermediate function
SHALL receive every required supported private machine value and SHALL forward
the same already-evaluated value at each direct call edge. A same-named caller,
parameter, or local binding in any frame SHALL remain isolated from `@`.

Within the existing hidden-capture group, forwarded context values SHALL retain
root declaration order. Every definition and call SHALL use matching exact LLVM
types under private `fastcc`, and LLVM SHALL own AMD64 physical placement. Full
O0 DWARF/GDB information SHALL expose every explicit parameter and forwarded
`@ member` value correctly in the active frame and in each suspended caller
frame. The backend MAY use target-aligned debug-only stack shadows to preserve
values across call-clobbered registers; those shadows SHALL NOT add
source-visible state or change execution semantics.

The lowering SHALL use no global context storage, namespace, capture, or
environment table, initializer replay, lookup, allocation, function pointer,
indirect call, foreign dependency, C/C++ runtime, other-language standard
library, public ABI, or native-ABI revision. Overload selection beyond
`TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001`, recursive forwarding beyond
`TOPAL-COMPILER-RECURSIVE-SCALAR-ENVIRONMENT-001`, local named Function
forwarding beyond `TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001`, anonymous
functions, aggregate environments beyond
`TOPAL-COMPILER-AGGREGATE-ENVIRONMENT-001`, otherwise unsupported context
members, escape, and public/library context environments remain deferred and
SHALL be rejected before artifact publication rather than assigned a
provisional environment ABI.

Future compiled-library metadata for a forwarding chain SHALL preserve the
canonical defining-context instance and source session, every selection and
call edge, callee identity and overload, member stable identity, captured
declaration position, visibility and declaration order, classifier and semantic
representation, capture order, lifetime and effects, and versioned target
adapter. Those facts SHALL remain independent of private capture names, LLVM
types or symbols, debug shadows, and target-specific physical argument
placement.

### TOPAL-COMPILER-FUNCTION-ROOT-DATA-001 — Private live-root data capture

For an ordinary root function called directly from the source entry frame, an
exact `root member` data selection SHALL resolve in the live source-session root
at the call position. The member MAY be declared after the function provided
its initializer has completed before the call. A same-named function parameter,
captured binding, or other lexical binding SHALL NOT intercept the qualified
selection. The initializer SHALL execute exactly once.

The checked frontend SHALL admit each referenced immutable member whose value
has an exact supported private machine representation and append those members
as explicit private capture arguments and parameters in root declaration order.
The call argument SHALL be the already-evaluated root storage value. The callee
SHALL bind it under the distinct source/debug name `root member`; definitions
and calls SHALL use identical exact LLVM types under `fastcc`, while LLVM owns
physical target placement. Full O0 DWARF/GDB information SHALL expose the
ordinary parameter and each root capture with its source classifier and value.

The lowering SHALL require no global root storage, namespace or capture table,
initializer replay, lookup, allocation, function pointer, indirect call,
foreign dependency, C/C++ runtime, other-language standard library, public ABI,
or native-ABI revision. Exact scalar forwarding between compiled functions is
governed by `TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001`. Aggregate values
beyond `TOPAL-COMPILER-AGGREGATE-ENVIRONMENT-001`, callable, Scope, Generator,
constraint, or evidence members; anonymous or escaping functions; root
selection forms beyond exact data selection; and public/library root
environments remain deferred and SHALL be rejected before artifact
publication.

Future compiled-library metadata for this boundary SHALL preserve the canonical
source-session namespace identity, selection and call positions, member stable
identity, visibility and declaration order, classifier and semantic
representation, capture order, lifetime and effects, and versioned target
adapter. Those facts SHALL remain independent of private capture names, LLVM
types or symbols, and target-specific physical argument placement.

### TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001 — Private live-root data forwarding

For a call from the source entry frame through a finite acyclic chain of
statically named ordinary root functions, the checked frontend SHALL compute
the transitive exact `root member` data selections before instantiating the
outer function. Each intermediate function SHALL receive every required
supported private machine value and SHALL forward that same already-evaluated
value at each direct call edge. An intermediate function need not select the
member itself. Resolution SHALL continue to use the live root snapshot at the
outer entry-frame call position, and a same-named explicit parameter or local
binding in any frame SHALL remain isolated from root qualification.

Within the existing hidden-capture group, forwarded root values SHALL retain
root declaration order. Every definition and call SHALL use matching exact LLVM
types under private `fastcc`, and LLVM SHALL own AMD64 physical placement.
Full O0 DWARF/GDB information SHALL expose every explicit parameter and
forwarded `root member` value correctly in the active frame and in each
suspended caller frame. The backend MAY use target-aligned debug-only stack
shadows to preserve values across call-clobbered registers; those shadows SHALL
NOT add source-visible state or change execution semantics.

The lowering SHALL use no global root storage, namespace or environment table,
initializer replay, lookup, allocation, function pointer, indirect call,
foreign dependency, C/C++ runtime, other-language standard library, public ABI,
or native-ABI revision. Overload selection beyond
`TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001`, recursive forwarding beyond
`TOPAL-COMPILER-RECURSIVE-SCALAR-ENVIRONMENT-001`, local named Function
forwarding beyond `TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001`, anonymous
functions, aggregate environments beyond
`TOPAL-COMPILER-AGGREGATE-ENVIRONMENT-001`, otherwise unsupported root members,
context (`@ member`) forwarding beyond
`TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001`, escape, and public/library root
environments remain deferred and SHALL be rejected before artifact publication
rather than assigned a provisional environment ABI.

Future compiled-library metadata for a forwarding chain SHALL preserve the
canonical source-session namespace identity, every selection and call edge,
callee identity and overload, member stable identity, visibility and
declaration order, classifier and semantic representation, capture order,
lifetime and effects, and versioned target adapter. Those facts SHALL remain
independent of private capture names, LLVM types or symbols, debug shadows, and
target-specific physical argument placement.

### TOPAL-COMPILER-RECURSIVE-SCALAR-ENVIRONMENT-001 — Proof-backed recursive scalar environments

When a recursive edge is independently admitted by
`TOPAL-COMPILER-FUNCTION-DECREASES-001`,
`TOPAL-COMPILER-RECURSION-INT-001`,
`TOPAL-COMPILER-RECURSION-INT-INCREASING-001`,
`TOPAL-COMPILER-RECURSION-NAT-001`,
`TOPAL-COMPILER-RECURSION-INT-MUTUAL-001`, or
`TOPAL-COMPILER-RECURSION-NAT-MUTUAL-001`, the checked frontend SHALL permit
that direct or mutual graph to carry the exact scalar hidden parameters already
governed by `TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001` and
`TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001`. Capture discovery SHALL NOT make
an otherwise unproven recursive edge admissible. Every graph member needed to
close a cycle SHALL receive the transitive union of represented captures, even
when a member does not select a value directly.

The source entry edge SHALL pass each defining-context value from the selecting
function's immutable declaration snapshot and each live-root value from the
outer call-position snapshot. Every recursive edge SHALL then forward its
current hidden parameters unchanged and in the established declaration order;
it SHALL NOT reload, re-evaluate, or look up a source value. A closing edge
SHALL target the already-reserved private symbol with the same exact LLVM
prototype and `fastcc` convention. LLVM SHALL own AMD64 physical placement, no
cycle member SHALL claim `norecurse`, and correctness SHALL hold at O0 without
inlining or tail-call conversion.

Full DWARF/GDB information SHALL expose each explicit parameter, `@ member`, and
`root member` accurately in the active recursive frame and every suspended
cycle frame. Target-aligned debug-only stack shadows MAY preserve call-clobbered
values without adding semantic state. The lowering SHALL add no global root or
context storage, environment/cycle table, lookup, initializer replay,
allocation, dispatcher, function pointer, indirect call, foreign dependency,
C/C++ runtime, other-language standard library, public ABI, or native-ABI
revision.

An unproven, incomplete, or mixed-proof cycle; overload selection beyond
`TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001`; unsupported captured representation;
local named Function recursion beyond
`TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001`; anonymous, nested, or escaping
recursion; and public/library recursive
environments remain deferred and SHALL fail before artifact publication.
Future compiled-library metadata SHALL preserve the recursion graph and member
identities, proof rule and evidence, every ordered call and capture edge,
canonical context/root instance, member identity and declaration position,
classifier and semantic representation, capture order/lifetime/effects, and a
versioned target adapter independently of private symbols, LLVM types, debug
shadows, and physical placement.

### TOPAL-COMPILER-AGGREGATE-ENVIRONMENT-001 — Private represented aggregate environments

The exact private environment mechanisms of
`TOPAL-COMPILER-FUNCTION-ROOT-DATA-001`,
`TOPAL-COMPILER-FUNCTION-ROOT-DATA-FORWARD-001`,
`TOPAL-COMPILER-CONTEXT-CAPTURE-001`, and
`TOPAL-COMPILER-CONTEXT-CAPTURE-FORWARD-001` SHALL admit a captured Tuple,
labeled Record, or nominal Sum when every component has an already-supported
exact private value representation and the complete aggregate contains no
Function value. This admission SHALL apply to direct entry calls, finite
acyclic statically named forwarding chains, and every direct or mutual
recursive edge independently admitted by the existing termination proofs. It
SHALL NOT make an otherwise unproven recursive graph admissible.

The source entry edge SHALL supply the already-evaluated immutable defining-
context or live-root aggregate. Each intermediate and recursive edge SHALL
forward the complete value unchanged, preserving Tuple position, Record source
field order and labels, and Sum nominal identity, active alternative, and
payload. Definition and call sites SHALL use the same exact compiler-private
LLVM aggregate type under `fastcc`. The compiler SHALL express the semantic
aggregate by value without `byval`, `sret`, `inalloca`, or `preallocated`
placement directives and SHALL leave AMD64 register/stack classification and
future target-specific physical placement to LLVM.

Full O0 DWARF/GDB information SHALL expose each aggregate under its source
capture name and classifier in active and suspended forwarding or recursive
frames. Target-aligned debug-only stack shadows MAY preserve call-clobbered
aggregate values without adding source-visible state. Correct execution and
debugging SHALL require no LLVM optimization.

The lowering SHALL add no global root/context storage, aggregate environment
object, environment or namespace table, lookup, initializer replay,
allocation, dispatcher, function pointer, indirect call, foreign dependency,
C/C++ runtime, other-language standard library, public ABI, or native-ABI
revision. Function-bearing aggregates; Scope, Generator, constraint, evidence,
static-only, opaque, or otherwise unsupported representations; overload
selection beyond `TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001`; local named Function
forwarding beyond `TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001`;
anonymous or escaping functions; and public/library aggregate environments
remain deferred and SHALL fail before artifact publication.

Future compiled-library metadata for such an environment SHALL preserve its
canonical context/root instance, source session, selection and call edges,
callee identity and overload, member stable identity and declaration position,
complete semantic classifier and recursive component structure, Tuple/Record
ordering and labels, Sum identity/alternative/payload, capture order, lifetime
and effects, and versioned target adapter. Those facts SHALL remain independent
of compiler-private capture names, LLVM aggregate types or symbols, debug
shadows, and physical argument placement.

### TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001 — Exact overload-selected private environments

For an ordinary statically named call through an overload set, the checked
frontend SHALL propagate defining-context and live-root captures from only the
source-ordered overload selected by the ordinary call rules when that selection
is determined before function instantiation by closed Unit, Boolean, Int,
Rational, or String literal arguments, explicit parameter classifiers, exact
Tuple or Record products of those values, or classifier-preserving `+`, `-`, or
`*` expressions over equal admitted classifiers. The same selected declaration
identity SHALL govern capture-graph
discovery and the eventual direct call. This admission SHALL compose with
direct entry, finite acyclic and cross-overload chains, represented aggregate
environments, and independently proven direct or mutual recursion.

Each overload SHALL retain its own exact ordered capture set. An unselected
overload SHALL neither add a hidden parameter nor suppress a capture required by
the selected overload. A selected cross-overload edge SHALL forward the current
exact hidden values to the selected target, and recursive edges SHALL continue
to require their independent termination proof. Definitions and calls SHALL use
matching compiler-private `fastcc` prototypes; LLVM SHALL own AMD64 physical
placement. No optimization SHALL be required for correctness.

If capture selection could change with retained value facts rather than the
admitted classifier/literal evidence—including `Int` to `Nat`, `String` to
`Character`, or exact `Rational` narrowing—or depends on a packaged, defaulted,
qualified, locally inferred, higher-order, anonymous, nested, dynamic, or
otherwise unresolved call, the compiler SHALL reject the program before
artifact publication whenever any candidate carries an environment. It SHALL
not union candidate capture sets or assign a provisional overload-call ABI. A
first-class retained overload set admitted by
`TOPAL-COMPILER-FUNCTION-ENVIRONMENT-BOUNDARY-001` SHALL instead transport the
complete deduplicated semantic environment required by that exact value while
retaining the declaration-specific schemas separately; after selection, its
direct call SHALL still receive only the selected declaration's capture vector.
An exact retained named alias admitted by
`TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001` SHALL use the same admitted
selection evidence and is not otherwise unresolved for this requirement.

Full O0 DWARF/GDB information SHALL expose the selected overload's explicit
parameters and exact source-named captures in active and suspended direct,
cross-overload, forwarding, and recursive frames. The lowering SHALL add no
global context/root state, overload or environment table, dispatcher, function
pointer, indirect call, lookup, replay, allocation, foreign dependency, C/C++
runtime, other-language standard library, public ABI, or native-ABI revision.

Future compiled-library metadata SHALL preserve canonical source-session and
context/root identities, call position, source-ordered overload set and exact
selected declaration identity, selection evidence and conversions, recursion
proof identity, member stable identity/declaration position, complete semantic
classifier and representation, ordered capture schema/lifetime/effects, and a
versioned target adapter. Those facts SHALL remain independent of private
capture names, LLVM types or symbols, debug shadows, and physical placement.

### TOPAL-COMPILER-LOCAL-FUNCTION-ENVIRONMENT-001 — Exact local named Function environments

Within an ordinary root-function invocation, a source-ordered local binding MAY
retain an already-visible named root Function or another exact local alias of
that same value and MAY apply the retained declaration snapshot under its local
name. An already-admitted non-escaping ordinary nested Function MAY likewise be
called directly or through an exact local alias. Before instantiating the outer
function, the checked frontend SHALL follow those exact alias and nested-call
edges when discovering defining-context and live-root selections. Lexical
shadowing or rebinding to any other value SHALL end that retained edge.

The retained root Function SHALL preserve its original name, staticness,
source-ordered visible overload snapshot, and declaration identities. An
overloaded alias call SHALL use the selection evidence admitted by
`TOPAL-COMPILER-OVERLOAD-ENVIRONMENT-001`; it SHALL propagate only the selected
declaration's capture set. A nested Function SHALL retain its own declaration
identity and ordinary lexical captures, but `@ member` and `root member` values
SHALL appear exactly once in its environment group rather than being duplicated
as lexical captures. Alias discovery SHALL NOT admit an otherwise unsupported
nested declaration, recursive edge, overload choice, capture representation,
or escape.

Every outer, nested, and selected target definition and direct call SHALL use
matching compiler-private `fastcc` prototypes. Existing represented scalar and
aggregate environment values SHALL be forwarded unchanged in declaration
order, LLVM SHALL own AMD64 physical placement, and correctness SHALL require no
optimization. Full O0 DWARF/GDB information SHALL expose each local Function
binding, selected source frame, explicit parameter, and exact source-named
capture in active and suspended frames.

The lowering SHALL add no closure or environment object, runtime Function
dispatch, function pointer, indirect call, global context/root state, lookup,
initializer replay, allocation, foreign dependency, C/C++ runtime,
other-language standard library, public ABI, or native-ABI revision. Exact
private Function boundaries carrying these environments are admitted by
`TOPAL-COMPILER-FUNCTION-ENVIRONMENT-BOUNDARY-001`. Escaping nested Functions;
recursive or overloaded nested Functions; opaque or dynamically selected
aliases; and public/library local-Function environments remain deferred and
SHALL fail before artifact publication.

Future compiled-library metadata SHALL preserve the canonical source session
and context/root identities, lexical invocation scope, alias binding stable
identity and declaration position, retained root Function and visible overload
snapshot or nested declaration path, selected declaration and call edge,
selection evidence/conversions, recursion proof identity, ordered capture
schema/classifiers/representations/lifetimes/effects, and a versioned target
adapter. Those facts SHALL remain independent of observation tags, private
capture names, LLVM types or symbols, debug shadows, and physical placement.

### TOPAL-COMPILER-FUNCTION-ENVIRONMENT-BOUNDARY-001 — Exact private Function-environment boundaries

Within one compilation unit, the checked frontend SHALL preserve an exact
Function value's defining-context, live-root, and ordinary lexical environment
when the value crosses a compiler-private scalar Function parameter or result,
or a represented Tuple or labeled Record parameter or result containing
Function fields. This admission covers exact named root Function values and
their retained visible overload sets, anonymous Functions constructed within
an ordinary root-function invocation, and already-admitted nested Functions.
Every Function leaf SHALL retain one exact callable identity and a
complete semantic capture schema; opaque or dynamically selected values SHALL
remain unsupported.

A retained named root overload set SHALL carry the complete deduplicated
defining-context and live-root environment required by its retained declarations
in root declaration order, with defining-context members before live-root
members. That transport environment is a property of the first-class value,
not a provisional overload-call ABI. Declaration identity, selection evidence,
and each declaration's capture schema SHALL remain distinct from the runtime
Function tag and from one another. Once an admitted ordinary application
selects a declaration, the direct target call SHALL receive exactly that
declaration's capture vector and no unselected capture. Selection that depends
on retained value facts, including narrowing from `Int` to `Nat`, SHALL be
rejected before artifact publication.

An anonymous or nested Function SHALL carry each referenced lexical,
defining-context, and live-root value exactly once. Forwarding through multiple
scalar or aggregate boundaries SHALL preserve the original immutable value and
callable identity without replaying an initializer or consulting a caller
frame. A private Function result SHALL return the Function representation and
its ordered captures together; a Function-containing aggregate result SHALL
associate each capture with its canonical Tuple-index, Record-label, Optional-
payload, or nominal-Sum-alternative path.
Named root Function results MAY cross further private boundaries. Anonymous
Functions SHALL remain non-escaping from the ordinary invocation that owns
their captured values. Nested Functions MAY escape only under
`TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`.

Every specialized boundary, result, and selected target SHALL use matching
compiler-private `fastcc` definitions and direct calls. LLVM SHALL own AMD64
aggregate classification and physical argument/result placement. Correctness
SHALL require no optimization. Full O0 DWARF/GDB information SHALL expose
source-visible Function parameters and fields, source-named captures, and the
selected, anonymous, or nested direct frame while active or suspended.

The lowering SHALL add no global context/root state, closure or environment
object, overload/environment table, runtime dispatcher, function pointer,
indirect call, lookup, initializer replay, allocation, foreign dependency,
C/C++ runtime, other-language standard library, public ABI, or native-ABI
revision. Sum representations outside `TOPAL-COMPILER-SUM-FUNCTION-001` or
other unsupported Function-containing representations;
escaping anonymous Function results or nested Function results outside
`TOPAL-COMPILER-NESTED-FUNCTION-ESCAPE-001`; recursive or overloaded nested
Functions; fact-dependent, opaque, or dynamic selection; and public/library
Function environments remain deferred and SHALL fail before artifact
publication.

Future compiled-library metadata SHALL preserve canonical source-session,
context/root, lexical-scope, callable, declaration, overload-snapshot, aggregate
path, and member stable identities; selection evidence and conversions;
recursion proof identity; complete semantic classifiers and representations;
ordered per-declaration and transported capture schemas, lifetimes, and effects;
and a versioned target adapter. Those facts SHALL remain independent of runtime
tags, compiler-private names, LLVM types or symbols, debug shadows, and physical
placement.

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

### TOPAL-COMPILER-SUM-EQUALITY-001 — Derived nominal Sum equality

The checked compiler model SHALL admit `=` and `!=` for two values of the same
exact nominal Union or positional Variant type exactly when every declared
payload provides an admitted canonical Equality operation. Payload-free
alternatives SHALL contribute Unit. Tuple and Record payloads and fields MAY
recursively contain an admitted Sum. A Sum with Function, Range, Result,
authority-bearing, or any other payload without admitted Equality SHALL be
rejected before LLVM lowering or artifact publication. Structurally identical
but nominally distinct Sum declarations SHALL have no shared equality overload.

Lowering SHALL compare tags first. Different tags SHALL produce false without
observing either payload. Equal tags SHALL select the one active alternative and
recursively compare only its payload; a payload-free alternative SHALL produce
true. Invalid private tags SHALL fail closed as unequal. LLVM `switch`, direct
recursive comparisons, and an `i1` `phi` MAY express this mandatory O0 control
flow. `!=` SHALL negate precisely the same equality result and observations.

The operation SHALL reuse the existing exact private tag-plus-payload aggregate,
matching `fastcc` prototypes, nominal DWARF type, and active-value GDB renderer,
while LLVM owns AMD64 register, stack, aggregate, and branch lowering. It SHALL
introduce no whole-storage comparison, inactive-field load, generic matcher,
Sum-equality runtime, allocation, callback, indirect dispatch, foreign
dependency, C/C++ runtime, other-language standard library, public Sum ABI, or
native-ABI revision. Future compiled-library metadata SHALL encode nominal
identity, ordered alternatives, payload schemas, canonical Equality evidence,
representation identity, and target adapters independently of private tags,
inactive storage, LLVM types, and physical placement. Recursive Sum
declarations, persistent/public storage, publication, and library adapters
remain deferred.

### TOPAL-COMPILER-RETURN-001 — Mandatory direct-return lowering

For every admitted direct `return` in a linear function body, the checked
compiler model SHALL evaluate and validate the return expression once, retain
all preceding statement effects in source order, and exclude every later
statement in that invocation from LLVM IR. This exclusion is mandatory
semantic lowering at `-O0`, not dead-code optimization. The backend SHALL use
the function's ordinary private result representation and truthful return-line
debug location. A root-level return SHALL be rejected.

### TOPAL-COMPILER-LEXICAL-RETURN-001 — Cleanup-free lexical return

An explicit `return` inside an admitted unconditional lexical block used
directly as a function-body statement, discard initializer, binding
initializer, or final expression SHALL complete the nearest enclosing ordinary
function. The checked compiler model SHALL evaluate preceding outer and inner
statements once in source order, validate the returned value against the
function result classifier, and exclude both the inner and outer unreachable
tails from generated IR at `-O0`.

The compiler MAY normalize that exit into the enclosing function's single
ordinary return after retaining the lexical block as its result expression.
Generated instructions before the exit SHALL retain the nested DWARF lexical
scope. This rule SHALL introduce no runtime control-flow object, unwinding
dependency, foreign runtime, C/C++ standard library, public ABI, or native-ABI
revision.

This increment admits only an unconditional block at a direct statement
boundary for which every exited scope is proven cleanup-free. Except for the
whole explicit-return operand admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-OPERAND-001` and the direct symbolic operand
admitted by `TOPAL-COMPILER-LEXICAL-RETURN-OPERATOR-001`, and the direct product
field admitted by `TOPAL-COMPILER-LEXICAL-RETURN-PRODUCT-001`, and the direct
named-call argument admitted by `TOPAL-COMPILER-LEXICAL-RETURN-CALL-001`, the
direct `Some` payload admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-001`, and the strict unary constructor
payload admitted by `TOPAL-COMPILER-LEXICAL-RETURN-CONSTRUCTOR-001`, and the
positional Variant payload admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-VARIANT-001`, and the direct `Character` operand
admitted by `TOPAL-COMPILER-LEXICAL-RETURN-CHARACTER-001`, and the direct named
constraint operand admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-CONSTRAINT-001`, and the direct named modular
operand admitted by `TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-001`, and the direct
modular-reduction operand admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-REDUCE-001`, and the direct unary List
collection source admitted by `TOPAL-COMPILER-LEXICAL-RETURN-COLLECT-001`, and
the direct unordered collection sources admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-UNORDERED-COLLECT-001`, and the direct Map
collection source admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-MAP-COLLECT-001`, a return-bearing block embedded
in another expression, conditional or callback, and any exit whose scope owns
generator close, resource, destructor, or other cleanup obligations SHALL
remain rejected until explicit exit-edge and cleanup lowering is implemented.

### TOPAL-COMPILER-LEXICAL-RETURN-OPERAND-001 — Return-expression lexical exit

An admitted cleanup-free lexical block used as the whole expression of an
explicit `return` MAY itself execute an explicit `return`. The inner return
SHALL complete the nearest enclosing ordinary function immediately. Its value
SHALL be evaluated and validated once, the outer return SHALL NOT create a
second semantic return decision, and every remaining statement in the lexical
block and function SHALL be excluded from generated IR at `-O0`.

The checked compiler model SHALL retain the lexical block as the function
result expression and the backend SHALL preserve its nested DWARF scope while
normalizing the exit into the function's existing single machine return.
Except for the direct symbolic operator operand admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-OPERATOR-001` and the direct product field
admitted by `TOPAL-COMPILER-LEXICAL-RETURN-PRODUCT-001`, and the direct
named-call argument admitted by `TOPAL-COMPILER-LEXICAL-RETURN-CALL-001`, the
direct `Some` payload admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-001`, and the strict unary constructor
payload admitted by `TOPAL-COMPILER-LEXICAL-RETURN-CONSTRUCTOR-001`, and the
positional Variant payload admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-VARIANT-001`, and the direct `Character` operand
admitted by `TOPAL-COMPILER-LEXICAL-RETURN-CHARACTER-001`, and the direct named
constraint operand admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-CONSTRAINT-001`, and the direct named modular
operand admitted by `TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-001`, and the direct
modular-reduction operand admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-REDUCE-001`, and the direct unary List
collection source admitted by `TOPAL-COMPILER-LEXICAL-RETURN-COLLECT-001`, and
the direct unordered collection sources admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-UNORDERED-COLLECT-001`, and the direct Map
collection source admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-MAP-COLLECT-001`, this rule admits no other
compound-expression, conditional, callback, generator, or cleanup-bearing
propagation and SHALL introduce no runtime control-flow object, unwind edge,
allocation, foreign dependency, C/C++ standard library, public ABI, or
native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-OPERATOR-001 — Symbolic-operand lexical exit

An admitted cleanup-free lexical block in the direct left or right operand
position of a symbolic operator MAY execute an explicit `return`. A left-side
exit SHALL occur before the operator or any right operand is evaluated. For a
right-side exit, the complete left application prefix SHALL be evaluated once
in source order, its abandoned value SHALL NOT be passed to the operator, and
the operator SHALL NOT be invoked. In both cases the returned value SHALL be
validated against the enclosing function result classifier and every remaining
statement in the lexical block and function SHALL be excluded from generated
IR at `-O0`.

The checked compiler model SHALL retain any evaluated left prefix as a private
exit sequence followed by the lexical block result. The backend SHALL emit that
prefix in the enclosing debug scope, retain the block's nested DWARF scope, and
normalize the exit into the function's existing single machine return. This
rule admits no constructor other than the direct `Some` payload covered by
`TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-001` and the strict unary constructors
covered by `TOPAL-COMPILER-LEXICAL-RETURN-CONSTRUCTOR-001`, packaged or nested
call argument, overloaded or indirect call, decision, callback, generator, or
cleanup-bearing propagation and SHALL introduce no runtime control-flow object,
unwind edge, allocation, foreign dependency, C/C++ standard library, public ABI,
or native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-PRODUCT-001 — Product-field lexical exit

An admitted cleanup-free lexical block used as a direct positional or labeled
product field MAY execute an explicit `return`. Fields preceding the exiting
field SHALL be evaluated exactly once in source order and their values SHALL be
abandoned. The product SHALL NOT be constructed, and the exiting field's
remaining statements, every later field, and the enclosing function tail SHALL
be excluded from generated IR at `-O0`. The returned value SHALL be validated
against the enclosing function result classifier rather than a product-field
or abandoned binding classifier.

The checked compiler model SHALL retain the evaluated field prefix as a private
exit sequence followed by the lexical block result. The backend SHALL emit each
prefix expression in the enclosing debug scope, retain the block's nested DWARF
scope, and normalize the exit into the function's existing single machine
return. This rule admits only a product that is itself at an admitted direct
statement or initializer boundary; a product nested in a constructor or call
argument, and every decision, callback, generator, or cleanup-bearing exit,
SHALL remain rejected. It SHALL introduce no runtime control-flow object,
unwind edge, aggregate allocation, foreign dependency, C/C++ standard library,
public ABI, or native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-CALL-001 — Named-call argument lexical exit

An admitted cleanup-free lexical block used as a direct argument of an
unshadowed named function with exactly one flat declaration MAY execute an
explicit `return`. The rule covers the sole argument of a unary prefix call and
either direct argument of a binary infix call. A left or sole-argument exit
SHALL occur before any other argument is evaluated. For a right-side exit, the
complete left application prefix SHALL be evaluated exactly once and its value
SHALL be abandoned. The named callee SHALL NOT be selected or invoked, and all
later arguments and function statements SHALL be excluded at `-O0`.

The returned value SHALL be validated against the enclosing function result
classifier rather than any callee parameter or abandoned binding classifier.
The checked compiler model SHALL retain an evaluated left prefix in the private
exit sequence and the backend SHALL preserve the returning block's nested DWARF
scope before using the enclosing function's existing single machine return.
Overloaded, bound, nested, qualified, packaged, defaulted, or constructor calls;
products nested in an argument; decisions; callbacks; generators; and exits
with cleanup obligations SHALL remain rejected. This rule SHALL introduce no
runtime control-flow object, indirect call, unwind edge, allocation, foreign
dependency, C/C++ standard library, public ABI, or native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-OPTIONAL-001 — Optional-constructor payload lexical exit

An admitted cleanup-free lexical block used as the direct payload of the
built-in unary `Some` constructor MAY execute an explicit `return`. The payload
return SHALL complete the nearest enclosing ordinary function before payload
classification or Optional construction. No `Some` value SHALL be constructed,
and the remaining payload statements and enclosing function tail SHALL be
excluded from generated IR at `-O0`.

The returned value SHALL be validated against the enclosing function result
classifier rather than the Optional payload or abandoned binding classifier.
The checked compiler model SHALL retain the returning block as the function
result, and the backend SHALL preserve its nested DWARF scope before using the
enclosing function's existing single machine return. Except for the strict
unary constructors admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-CONSTRUCTOR-001`, other constructors, products or
other expressions nested in the payload position, decisions, callbacks,
generators, and exits with cleanup obligations SHALL remain rejected. This rule
SHALL introduce no runtime control-flow object, Optional allocation, indirect
call, unwind edge, foreign dependency, C/C++ standard library, public ABI, or
native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-CONSTRUCTOR-001 — Strict unary-constructor payload lexical exit

An admitted cleanup-free lexical block used as the direct operand of the
built-in unary `String`, `Int`, `Nat`, or `Rational` constructor, or as the
direct payload of an already-declared payload-bearing Union alternative, MAY
execute an explicit `return`. The operand return SHALL complete the nearest
enclosing ordinary function before conversion, payload classification, or
constructor selection. No constructed value, nominal Union tag, or abandoned
binding SHALL be emitted, and remaining operand statements and the enclosing
function tail SHALL be excluded from generated IR at `-O0`.

The returned value SHALL be validated against the enclosing function result
classifier rather than the constructor operand, Union payload, or abandoned
binding classifier. The checked compiler model SHALL retain the returning block
as the function result, and the backend SHALL preserve its nested DWARF scope
before using the enclosing function's existing single machine return.
Except for the positional Variant form admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-VARIANT-001` and the direct `Character` form
admitted by `TOPAL-COMPILER-LEXICAL-RETURN-CHARACTER-001`, and the direct named
Int-constraint form admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-CONSTRAINT-001`, and the direct named modular
form admitted by `TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-001`, and the direct
modular-reduction operand admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-REDUCE-001`, and the direct unary List
collection source admitted by `TOPAL-COMPILER-LEXICAL-RETURN-COLLECT-001`, and
the direct unordered collection sources admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-UNORDERED-COLLECT-001`, and the direct Map
collection source admitted by
`TOPAL-COMPILER-LEXICAL-RETURN-MAP-COLLECT-001`, other constraint, modular, or
collection forms, qualified and any other constructor forms; products or other
expressions nested in the operand; decisions; callbacks; generators; and exits
with cleanup obligations SHALL remain rejected. This rule SHALL introduce no
runtime control-flow object, constructor-specific allocation, indirect call,
unwind edge, foreign dependency, C/C++ standard library, public ABI, or
native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-VARIANT-001 — Positional Variant payload lexical exit

An admitted cleanup-free lexical block used as the direct payload of an
already-declared positional Variant constructor `Type at index payload` MAY
execute an explicit `return` only when `index` names an existing alternative.
The declared Variant identity and index SHALL be checked before entering the
payload. The payload return SHALL then complete the nearest enclosing ordinary
function before payload classification or Variant construction. No Variant tag,
payload aggregate, or abandoned binding SHALL be emitted, and remaining
payload statements and the enclosing function tail SHALL be excluded from
generated IR at `-O0`.

The returned value SHALL be validated against the enclosing function result
classifier rather than the selected Variant payload or abandoned binding
classifier. The checked compiler model SHALL retain the returning block as the
function result, and the backend SHALL preserve its nested DWARF scope before
using the enclosing function's existing single machine return. Invalid,
undeclared, qualified, nested-payload, decision, callback, generator, and
cleanup-bearing forms SHALL remain fail-closed under their existing diagnostic
contracts. This rule SHALL introduce no runtime control-flow object,
Variant-specific allocation, indirect call, unwind edge, foreign dependency,
C/C++ standard library, public ABI, or native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-CHARACTER-001 — Character-constraint operand lexical exit

An admitted cleanup-free lexical block used as the direct operand of the
built-in `Character` constructor MAY execute an explicit `return`. Selection of
the built-in constructor identity SHALL precede the operand. The operand return
SHALL then complete the nearest enclosing ordinary function before pinned-Unicode
Character constraint validation. No Character evidence or abandoned binding
SHALL be emitted, and remaining operand statements and the enclosing function
tail SHALL be excluded from generated IR at `-O0`.

The returned value SHALL be validated against the enclosing function result
classifier rather than `Character` or an abandoned binding classifier. The
checked compiler model SHALL retain the returning block as the function result,
and the backend SHALL preserve its nested DWARF scope before using the enclosing
function's existing single machine return. Qualified, nested-operand, decision,
callback, generator, and cleanup-bearing forms SHALL remain fail-closed under
their existing diagnostic contracts. This rule SHALL introduce no runtime
control-flow object, Character-validation routine, allocation, indirect call,
unwind edge, foreign dependency, C/C++ standard library, public ABI, or
native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-CONSTRAINT-001 — Named-constraint operand lexical exit

An admitted cleanup-free lexical block used as the direct operand of an
already-declared named Int constraint MAY execute an explicit `return`. The
compiler SHALL validate the constraint identity, Int base, and declaration
order before entering the operand. The operand return SHALL then complete the
nearest enclosing ordinary function before predicate evaluation or refined
evidence construction. No predicate branch, refined evidence, or abandoned
binding SHALL be emitted, and remaining operand statements and the enclosing
function tail SHALL be excluded from generated IR at `-O0`.

The returned value SHALL be validated against the enclosing function result
classifier rather than the constraint or an abandoned binding classifier. The
checked compiler model SHALL retain the returning block as the function result,
and the backend SHALL preserve its nested DWARF scope before using the enclosing
function's existing single machine return. Forward, unknown, non-Int,
dynamically selected, qualified, nested-operand, decision, callback, generator,
and cleanup-bearing forms SHALL remain fail-closed under their existing
diagnostic contracts. This rule SHALL introduce no runtime control-flow object,
constraint validation, allocation, indirect call, unwind edge, foreign
dependency, C/C++ standard library, public ABI, or native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-001 — Named-modular operand lexical exit

An admitted cleanup-free lexical block used as the direct operand of an
already-declared named `ModNat` or `ModInt` type MAY execute an explicit
`return`. The compiler SHALL validate the modular identity and declaration
order before entering the operand. The operand return SHALL then complete the
nearest enclosing ordinary function before Int classification, range
validation, or nominal-value construction. No range comparison, dynamic Result,
nominal value, or abandoned binding SHALL be emitted, and remaining operand
statements and the enclosing function tail SHALL be excluded from generated IR
at `-O0`.

The returned value SHALL be validated against the enclosing function result
classifier rather than the modular type or an abandoned binding classifier. The
checked compiler model SHALL retain the returning block as the function result,
and the backend SHALL preserve its nested DWARF scope before using the enclosing
function's existing single machine return. Forward, unknown, dynamically
selected, qualified, explicit-reduction forms outside
`TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-REDUCE-001`, nested-operand, decision,
callback, generator, and cleanup-bearing forms SHALL remain fail-closed under
their existing diagnostic contracts. This rule SHALL introduce no runtime
control-flow object, modular validation or reduction, allocation, indirect call,
unwind edge, foreign dependency, C/C++ standard library, public ABI, or
native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-MODULAR-REDUCE-001 — Modular-reduction operand lexical exit

An admitted cleanup-free lexical block used as the direct left operand of
`operand modulo Name` MAY execute an explicit `return` when `Name` is an
already-declared named `ModNat` or `ModInt` type. The compiler SHALL validate
the exact modular target and declaration order before entering the operand. The
operand return SHALL then complete the nearest enclosing ordinary function
before Int classification or modular reduction. No subtract/modulo/add
reduction chain, nominal value, or abandoned binding SHALL be emitted, and
remaining operand statements and the enclosing function tail SHALL be excluded
from generated IR at `-O0`.

The returned value SHALL be validated against the enclosing function result
classifier rather than the modular type or an abandoned binding classifier. The
checked compiler model SHALL retain the returning block as the function result,
and the backend SHALL preserve its nested DWARF scope before using the enclosing
function's existing single machine return. Forward, unknown, dynamically
selected, qualified, nested-operand, decision, callback, generator, and
cleanup-bearing forms SHALL remain fail-closed under their existing diagnostic
contracts. This rule SHALL introduce no runtime control-flow object, modular
validation or reduction, allocation, indirect call, unwind edge, foreign
dependency, C/C++ standard library, public ABI, or native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-COLLECT-001 — Unary List-collection source lexical exit

An admitted cleanup-free lexical block used as the direct source of the
built-in unary `collect source` operation MAY execute an explicit `return`.
Selection of the exact built-in operation SHALL precede the source. The source
return SHALL then complete the nearest enclosing ordinary function before
finite-traversal classification, generator consumption, or List
materialization. No List node, consumption action, or abandoned binding SHALL
be emitted, and remaining source statements and the enclosing function tail
SHALL be excluded from generated IR at `-O0`.

The returned value SHALL be validated against the enclosing function result
classifier rather than a List or abandoned binding classifier. The checked
compiler model SHALL retain the returning block as the function result, and the
backend SHALL preserve its nested DWARF scope before using the enclosing
function's existing single machine return. Except for `collect-set` and
`collect-bag` covered by
`TOPAL-COMPILER-LEXICAL-RETURN-UNORDERED-COLLECT-001`, infix Array or String
collection, and the `collect-map` form covered by
`TOPAL-COMPILER-LEXICAL-RETURN-MAP-COLLECT-001`, qualified, nested-source,
decision, callback, generator-cleanup, and other cleanup-bearing forms SHALL
remain fail-closed under their existing diagnostic contracts. This rule SHALL
introduce no runtime control-flow object, collection materialization,
allocation, indirect call, unwind edge, foreign dependency, C/C++ standard
library, public ABI, or native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-UNORDERED-COLLECT-001 — Unordered-collection source lexical exit

An admitted cleanup-free lexical block used as the direct source of the exact
built-in `collect-set source` or `collect-bag source` operation MAY execute an
explicit `return`. Selection of the exact built-in operation SHALL precede the
source. The source return SHALL then complete the nearest enclosing ordinary
function before finite-List classification, equality-dependent duplicate
coalescing, multiplicity accumulation, or Set or Bag materialization. No Set or
Bag node, comparison, consumption action, or abandoned binding SHALL be
emitted, and remaining source statements and the enclosing function tail SHALL
be excluded from generated IR at `-O0`.

The returned value SHALL be validated against the enclosing function result
classifier rather than an unordered collection or abandoned binding
classifier. The checked compiler model SHALL retain the returning block as the
function result, and the backend SHALL preserve its nested DWARF scope before
using the enclosing function's existing single machine return. Except for
`collect-map` covered by `TOPAL-COMPILER-LEXICAL-RETURN-MAP-COLLECT-001`,
qualified, nested-source, decision, callback, cleanup-bearing, and other
collection forms SHALL remain fail-closed under their existing diagnostic
contracts. This rule SHALL introduce no runtime control-flow object, collection
materialization, comparison, allocation, indirect call, unwind edge, foreign
dependency, C/C++ standard library, public ABI, or native-ABI revision.

### TOPAL-COMPILER-LEXICAL-RETURN-MAP-COLLECT-001 — Map-collection source lexical exit

An admitted cleanup-free lexical block used as the direct source of the exact
built-in `collect-map source resolving policy` operation MAY execute an
explicit `return` when `policy` is exactly `reject`, `keep-first`, or
`keep-last`. Selection of the operation, `resolving` clause, and valid collision
policy SHALL precede the source. The source return SHALL then complete the
nearest enclosing ordinary function before pair classification, key equality,
collision resolution, or Map materialization. No Map node, comparison,
collision action, or abandoned binding SHALL be emitted, and remaining source
statements and the enclosing function tail SHALL be excluded from generated IR
at `-O0`.

The returned value SHALL be validated against the enclosing function result
classifier rather than a Map or abandoned binding classifier. The checked
compiler model SHALL retain the returning block as the function result, and the
backend SHALL preserve its nested DWARF scope before using the enclosing
function's existing single machine return. Invalid policies, qualified,
nested-source, decision, callback, cleanup-bearing, and other collection forms
SHALL remain fail-closed under their existing diagnostic contracts. This rule
SHALL introduce no runtime control-flow object, collection materialization,
comparison, allocation, indirect call, unwind edge, foreign dependency, C/C++
standard library, public ABI, or native-ABI revision.

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

### TOPAL-COMPILER-LIST-EFFECT-CORE-001 — Ordinary canonical-empty Effect Lists

The compiler SHALL extend `TOPAL-COMPILER-LIST-EFFECT-001` with ordinary
private parameter/result/package and recursively admitted Tuple/Record passage,
structural equality and inequality, complete `Empty`/`Entry (first, rest)`
decisions, entry count, emptiness, canonical display, and debugging. Every
operand SHALL evaluate once in source order. Within this increment every entry
is the sealed canonical empty Effect row, so structural equality SHALL compare
List lengths without performing an effect.

On Linux x86-64, equality and counting MAY use a conditional finite
nonrecursive fragment over the existing private 16-byte Effect node. Decisions
SHALL load the exact sealed i8 carrier and emptiness SHALL be a null test.
Correctness SHALL not depend on optimization. LLVM SHALL select physical AMD64
placement for exact private prototypes and aggregates; DWARF/GDB SHALL preserve
the `List Effect` identity and safely render every entry.

This rule SHALL NOT define representation or equality for future nonempty
effect-row identities, perform an effect, or add a runtime tag, generic/public/
foreign/library node ABI, foreign allocator, C/C++ runtime, other-language
standard library, or native-ABI revision. Other List operations, reclamation,
and element classifiers remain separately governed. Future library metadata
SHALL encode effect-row identity/evidence, representation, ownership, lifetime,
effects, and versioned target adapters independently of private offsets, helper
names, LLVM types/symbols, debug shadows, and physical placement.

### TOPAL-COMPILER-LIST-COMPARISON-CORE-001 — Ordinary immutable Comparison Lists

The compiler SHALL admit contextual `Empty` and `Entry` construction for
`List Comparison`, immutable binding, ordinary private parameters/results,
package and recursively admitted Tuple/Record fields, structural equality and
inequality, complete decisions, entry count, emptiness, canonical display, and
debugging. Every operand SHALL evaluate once in source order. Each entry SHALL
retain exactly one of `Less`, `Equal`, or `Greater`; equality SHALL compare List
length and corresponding alternatives in order without mutating either input.

On Linux x86-64, `Empty` MAY be a null private pointer and `Entry` MAY use an
immutable 16-byte node containing the existing i32 Comparison carrier and the
remaining pointer. A conditional finite nonrecursive fragment MAY implement
equality and counting and SHALL remain correct at O0. LLVM SHALL select
physical AMD64 placement for exact private prototypes and aggregates. DWARF
and the bundled GDB renderer SHALL preserve the `List Comparison` identity and
validate every closed alternative.

This rule SHALL add no runtime type tag, type-erased generic List, public,
foreign, serialized, or compiled-library node/enum ABI, foreign allocator,
C/C++ runtime, other-language standard library, or native-ABI revision. Other
List operations, reclamation, and element classifiers remain separately
governed. Future library metadata SHALL encode the semantic alternative
mapping, representation and ownership, lifetime, effects, and versioned target
adapters independently of private offsets, helper names, LLVM types/symbols,
debug shadows, and physical placement.

### TOPAL-COMPILER-LIST-ERROR-CODE-CORE-001 — Ordinary arithmetic ErrorCode Lists

The compiler SHALL admit contextual `Empty` and `Entry` construction for
`List ErrorCode` whose entries belong to the closed
`lang arithmetic ArithmeticErrorCode` vocabulary, immutable binding, ordinary
private parameters/results, package and recursively admitted Tuple/Record
fields, structural equality and inequality, complete decisions, entry count,
emptiness, canonical display, and debugging. Operands SHALL evaluate once in
source order. Equality SHALL compare List length and corresponding qualified
code identities without mutating either input.

On Linux x86-64, `Empty` MAY be null and `Entry` MAY use an immutable 16-byte
node containing the existing i32 arithmetic-code carrier and remaining pointer.
A conditional finite nonrecursive fragment MAY implement equality and counting
and SHALL remain correct at O0. LLVM SHALL select physical AMD64 placement for
exact private prototypes and aggregates. DWARF and the bundled GDB renderer
SHALL preserve the complete arithmetic vocabulary and validate all four codes.

This rule SHALL NOT assign the same tags or representation to another
ErrorCode vocabulary by coincidence or add a runtime type tag, generic/public/
foreign/serialized/library node or enum ABI, foreign allocator, C/C++ runtime,
other-language standard library, or native-ABI revision. Other List operations,
reclamation, and vocabularies remain separately governed. Future library
metadata SHALL encode canonical vocabulary/alternative identities,
representation and ownership, lifetime, effects, and versioned target adapters
independently of private offsets, helpers, LLVM types/symbols, debug shadows,
and physical placement.

### TOPAL-COMPILER-LIST-UNIT-CORE-001 — Ordinary immutable Unit Lists

The compiler SHALL admit contextual `Empty` and `Entry` construction for
`List Unit`, immutable binding, ordinary private parameters/results, package
and recursively admitted Tuple/Record fields, structural equality and
inequality, complete decisions, entry count, emptiness, canonical display, and
debugging. Every entry SHALL retain the sole Unit value `()`, and operands SHALL
evaluate once in source order. Equality SHALL compare List length without
inventing payload information or mutating either input.

On Linux x86-64, `Empty` MAY be null and `Entry` MAY use an immutable 16-byte
node with a validated zero i8 carrier and remaining pointer. A conditional
finite nonrecursive fragment MAY implement length equality and counting and
SHALL remain correct at O0. An ordinary Unit function result SHALL retain the
existing private void convention; Unit fields in private aggregates MAY retain
their existing i8 carrier. LLVM SHALL select physical AMD64 placement. DWARF
and the bundled GDB renderer SHALL preserve `Unit`, `List Unit`, and every
entry/empty distinction.

This rule SHALL NOT reinterpret Unit as `Completed` or `Effect`, introduce
completion evidence or payload state, or add a runtime tag, generic/public/
foreign/serialized/library node ABI, foreign allocator, C/C++ runtime,
other-language standard library, or native-ABI revision. Other List operations,
reclamation, and element classifiers remain separately governed. Future
library metadata SHALL encode Unit identity, representation and ownership,
lifetime, effects, and versioned target adapters independently of private
offsets, helper names, LLVM types/symbols, debug shadows, and placement.

### TOPAL-COMPILER-LIST-COMPLETED-CORE-001 — Ordinary immutable Completed Lists

The compiler SHALL admit contextual `Empty` and `Entry` construction for
`List Completed`, immutable binding, ordinary private parameters/results,
package and recursively admitted Tuple/Record fields, structural equality and
inequality, complete decisions, entry count, emptiness, canonical display, and
debugging. Each entry SHALL retain the sole `Completed` value as explicit
completion evidence. Operands SHALL evaluate once in source order, and equality
SHALL compare List length without mutating either input.

On Linux x86-64, `Empty` MAY be null and `Entry` MAY use an immutable 16-byte
node containing the existing validated-zero i8 Completed carrier and remaining
pointer. A conditional finite nonrecursive fragment MAY implement length
equality and counting and SHALL remain correct at O0. Private Completed
parameters, results, and aggregate fields SHALL retain exact typed i8
prototypes while LLVM selects physical AMD64 placement. DWARF and the bundled
GDB renderer SHALL preserve `Completed`, `List Completed`, and every
entry/empty distinction.

This rule SHALL NOT reinterpret Completed as Unit or Effect, discard its
completion-evidence meaning, or add payload state, a runtime tag, generic/
public/foreign/serialized/library node ABI, foreign allocator, C/C++ runtime,
other-language standard library, or native-ABI revision. Other List operations,
reclamation, and element classifiers remain separately governed. Future
library metadata SHALL encode completion semantics, representation and
ownership, lifetime, effects, and versioned target adapters independently of
private offsets, helper names, LLVM types/symbols, debug shadows, and placement.

### TOPAL-COMPILER-LIST-TYPE-CORE-001 — Ordinary immutable fundamental Type Lists

The compiler SHALL admit contextual `Empty` and `Entry` construction for
`List Type` containing the seven fundamental Type values, immutable binding,
ordinary private parameters/results, package and recursively admitted
Tuple/Record fields, structural equality and inequality, complete decisions,
entry count, emptiness, canonical display, and debugging. Equality SHALL compare
canonical type identity in order and stop at the first mismatch without
mutating either input. Operands SHALL evaluate once in source order.

On Linux x86-64, `Empty` MAY be null and `Entry` MAY use an immutable 16-byte
node containing the existing closed i32 fundamental-Type carrier and remaining
pointer. A conditional finite nonrecursive fragment MAY implement exact
identity equality and counting and SHALL remain correct at O0. Private Type
parameters, results, and aggregate fields SHALL retain exact typed i32
prototypes while LLVM selects physical AMD64 placement. DWARF and the bundled
GDB renderer SHALL preserve `Type`, `List Type`, and all seven canonical
identities.

This rule SHALL NOT turn the carrier into a host-language or LLVM descriptor,
admit additional Type identities, or add a List runtime tag, type-erased
generic/public/foreign/serialized/library node ABI, foreign allocator, C/C++
runtime, other-language standard library, or native-ABI revision. Other List
operations, reclamation, and element classifiers remain separately governed.
Future library metadata SHALL encode the canonical fundamental identity set,
representation and ownership, lifetime, effects, and versioned target adapters
independently of private numeric mappings, offsets, helper names, LLVM types/
symbols, debug shadows, and placement.

### TOPAL-COMPILER-LIST-OPTIONAL-INT-CORE-001 — Ordinary Optional Int Lists

The compiler SHALL admit empty/nonempty `List Optional Int` construction,
immutable binding, private parameter/result/package and Tuple/Record passage,
derived structural equality, complete decisions, count, emptiness, display, and
debugging. List equality SHALL preserve `None`/`Some`, delegate `Some` payloads
to exact Int equality, stop at the first mismatch, and remain correct at O0.

On Linux x86-64, an Entry MAY use an immutable 16-byte node containing the
existing Optional-header pointer and remaining pointer. LLVM SHALL own physical
AMD64 placement. This rule SHALL add no flattened tag, copied Int, type-erased
container, public/foreign/library ABI, foreign allocator, C/C++ runtime,
other-language standard library, or native-ABI revision. Future metadata SHALL
encode both container classifiers, Optional alternatives, representation/
ownership/lifetime/effects, and target adapters independently of private
headers, offsets, LLVM types, helpers, and debug shadows.

### TOPAL-COMPILER-LIST-OPTIONAL-RATIONAL-CORE-001 — Ordinary Optional Rational Lists

The compiler SHALL admit empty/nonempty `List Optional Rational` construction,
immutable binding, private parameter/result/package and Tuple/Record passage,
derived structural equality, complete decisions, count, emptiness, display, and
debugging. List equality SHALL preserve `None`/`Some`, delegate `Some` payloads
to exact Rational equality, stop at the first mismatch, and remain correct at
O0, including canonical finite and infinite Rational values.

On Linux x86-64, an Entry MAY use an immutable 16-byte node containing the
existing Optional-header pointer and remaining pointer. LLVM SHALL own physical
AMD64 placement. This rule SHALL add no flattened tag, copied Rational, type-
erased container, public/foreign/library ABI, foreign allocator, C/C++ runtime,
other-language standard library, or native-ABI revision. Future metadata SHALL
encode all three classifier layers, Optional alternatives, representation/
ownership/lifetime/effects, and target adapters independently of private
headers, offsets, LLVM types, helpers, and debug shadows.

### TOPAL-COMPILER-LIST-OPTIONAL-STRING-CORE-001 — Ordinary Optional String Lists

The compiler SHALL admit empty/nonempty `List Optional String` construction,
immutable binding, private parameter/result/package and Tuple/Record passage,
derived structural equality, complete decisions, count, emptiness, display, and
debugging. List equality SHALL preserve `None`/`Some`, delegate `Some` payloads
through Optional-String equality to exact String comparison, stop at the first
mismatch, and remain correct at O0 for arbitrary admitted UTF-8 contents.

On Linux x86-64, an Entry MAY use an immutable 16-byte node containing the
existing Optional-header pointer and remaining pointer. LLVM SHALL own physical
AMD64 placement. This rule SHALL add no flattened tag, copied String descriptor,
type-erased container, public/foreign/library ABI, foreign allocator, C/C++
runtime, other-language standard library, or native-ABI revision. Future
metadata SHALL encode all three classifier layers, Optional alternatives,
representation/ownership/lifetime/effects, and target adapters independently of
private headers, descriptors, offsets, LLVM types, helpers, and debug shadows.

### TOPAL-COMPILER-LIST-ENUM-CORE-001 — Ordinary payload-free nominal Enum Lists

For every admitted payload-free nominal Enum, the compiler SHALL admit
contextual `Empty` and `Entry` construction, immutable binding, ordinary private
parameters/results, package and recursively admitted Tuple/Record fields,
structural equality and inequality, complete decisions, entry count, emptiness,
canonical display, and debugging for its List type. Every value SHALL retain
the enum declaration identity and source alternative. Equality SHALL apply only
to Lists with the same nominal element type, compare alternatives in order, and
stop at the first mismatch without mutation. Operands SHALL evaluate once in
source order.

On Linux x86-64, `Empty` MAY be null and `Entry` MAY use an immutable 16-byte
node containing the existing declaration-ordered i32 enum carrier and remaining
pointer. A conditional finite nonrecursive fragment MAY compare tags and count
nodes and SHALL remain correct at O0; checking SHALL establish equal nominal
element identity before that common fragment is selected. Private enum
parameters, results, and aggregate fields SHALL retain exact typed i32
prototypes while LLVM selects physical AMD64 placement. DWARF and the bundled
GDB renderer SHALL preserve the nominal enum, its alternatives, and List shape.

This rule SHALL NOT exchange tags between distinct enum declarations or expose
them as a portable foreign enum. It SHALL add no List runtime tag, type-erased
generic/public/foreign/serialized/library node ABI, foreign allocator, C/C++
runtime, other-language standard library, or native-ABI revision. Other List
operations, reclamation, and element classifiers remain separately governed.
Future library metadata SHALL encode canonical enum declaration/alternative
identities, representation and ownership, lifetime, effects, and versioned
target adapters independently of private numeric mappings, offsets, helper
names, LLVM types/symbols, debug shadows, and placement.

### TOPAL-COMPILER-LIST-MODULAR-CORE-001 — Ordinary nominal modular Lists

For every admitted root nominal modular type, the compiler SHALL admit
contextual `Empty` and `Entry` construction, immutable binding, ordinary private
parameters/results, packages, and recursively admitted Tuple/Record fields,
structural equality and inequality, complete decisions, entry count, emptiness,
canonical display, and debugging for its List type. Every entry SHALL retain the
exact modular declaration and canonical representative. Equality SHALL apply
only to Lists with the same nominal element type, compare exact representatives
in order, and stop at the first mismatch without mutation. Operands SHALL
evaluate once in source order.

On Linux x86-64, `Empty` MAY be null and `Entry` MAY use an immutable 16-byte
node containing the existing canonical arbitrary-precision Int pointer and
remaining pointer. A conditional finite nonrecursive fragment MAY compare
representatives with exact Int comparison and count nodes and SHALL remain
correct at O0; checking SHALL establish equal nominal element identity before
that common fragment is selected. Private modular parameters, results, and
Tuple/Record fields SHALL retain exact pointer prototypes while LLVM selects
physical AMD64 placement. DWARF and the bundled GDB renderer SHALL preserve the
modular declaration, bounds-derived identity, canonical value, and List shape.

This rule SHALL NOT narrow, copy, truncate, wrap, or re-reduce an already
canonical representative, exchange values between modular declarations, or
expose the carrier as a machine integer or portable foreign number. It SHALL
add no List runtime tag, type-erased generic/public/foreign/serialized/library
node ABI, foreign allocator, C/C++ runtime, other-language standard library, or
native-ABI revision. Other List operations, reclamation, and other element
classifiers remain separately governed. Future library metadata SHALL encode
canonical declaration identity, signedness and exact bounds, representation and
ownership, lifetime, effects, and versioned target adapters independently of
private Int/List layouts, offsets, helper names, LLVM types/symbols, debug
shadows, and placement.

### TOPAL-COMPILER-LIST-BOOLEAN-001 — Ordinary immutable Boolean Lists

The compiler SHALL admit contextual `Empty` and `Entry` construction for
`List Boolean`, immutable binding, ordinary private parameters and results,
package fields, recursively admitted Tuple/Record fields, structural equality
and inequality, complete `Empty`/`Entry (first, rest)` decisions, entry count,
emptiness, canonical display, and debugging. Construction and calls SHALL
evaluate their operands once in source order. Equality SHALL compare entry
count and corresponding Boolean values in order, stopping at the first
mismatch, without mutating either input.

On Linux x86-64, `Empty` MAY remain a null private pointer and each `Entry` MAY
use an immutable naturally aligned 16-byte node containing the i1 Boolean at
offset zero and the remaining-node pointer at offset eight. Padding SHALL NOT
be source state. Equality and counting SHALL use finite nonrecursive control
flow whose correctness at O0 does not depend on optimization. Private
definitions, calls, returns, and containing private aggregates SHALL use exact
pointer-bearing prototypes with physical AMD64 placement selected by LLVM.
DWARF and the bundled GDB renderer SHALL preserve and safely render the source
`List Boolean` identity and entries.

This rule SHALL add no runtime type tag, type-erased generic List, public,
foreign, serialized, or compiled-library node ABI, foreign allocator, C/C++
runtime, other-language standard library, or native-ABI revision. Projections,
insertion, concatenation, reversal, removal, range selection, traversal,
higher-order transforms, reclamation beyond process lifetime, and other
element classifiers remain governed by separate rules. Future compiled-library
metadata SHALL encode element classification, representation and ownership,
lifetime, effects, and versioned target adapters independently of node offsets,
private helper names, LLVM types/symbols, debug shadows, and physical placement.

### TOPAL-COMPILER-LIST-STRING-CORE-001 — Ordinary immutable String Lists

The compiler SHALL admit contextual `Empty` and `Entry` construction for
`List String`, immutable binding, ordinary private parameters and results,
package fields, recursively admitted Tuple/Record fields, structural equality
and inequality, complete `Empty`/`Entry (first, rest)` decisions, entry count,
emptiness, canonical display, and debugging. Construction and calls SHALL
evaluate their operands once in source order. Equality SHALL compare length and
corresponding entries in order through canonical preserved-sequence String
equality, stopping at the first mismatch without mutating either input.

On Linux x86-64, `Empty` MAY remain a null private pointer and each `Entry` MAY
use an immutable 16-byte node containing the existing String descriptor pointer
and the remaining-node pointer. Equality and counting SHALL use finite
nonrecursive control flow whose correctness at O0 does not depend on
optimization. Private definitions, calls, returns, and containing private
aggregates SHALL use exact pointer-bearing prototypes with physical AMD64
placement selected by LLVM. DWARF and the bundled GDB renderer SHALL preserve
and safely render the source `List String` identity, Unicode contents, and
String delimiters.

This rule SHALL add no copied String representation, host text API, runtime
type tag, type-erased generic List, public, foreign, serialized, or
compiled-library node ABI, foreign allocator, C/C++ runtime, other-language
standard library, or native-ABI revision. Projections, insertion,
concatenation, reversal, removal, range selection, traversal, higher-order
transforms, reclamation beyond process lifetime, and other element classifiers
remain governed by separate rules. Future compiled-library metadata SHALL
encode element classification, String and node representation/ownership,
lifetime, effects, and versioned target adapters independently of node offsets,
private helper names, LLVM types/symbols, debug shadows, and physical placement.

### TOPAL-COMPILER-LIST-CHARACTER-CORE-001 — Ordinary immutable Character Lists

The compiler SHALL admit contextual `Empty` and `Entry` construction for
`List Character`, immutable binding, ordinary private parameters and results,
package fields, recursively admitted Tuple/Record fields, structural equality
and inequality, complete `Empty`/`Entry (first, rest)` decisions, entry count,
emptiness, canonical display, and debugging. Construction and calls SHALL
evaluate their operands once in source order. Every stored entry SHALL retain
valid Character evidence and its complete preserved Unicode scalar sequence.
Equality SHALL compare length and corresponding entries in order through the
canonical equality derived from String, stopping at the first mismatch without
normalizing or mutating either input.

On Linux x86-64, `Empty` MAY remain a null private pointer and each `Entry` MAY
use an immutable 16-byte node containing the existing constrained String
descriptor pointer and the remaining-node pointer. Equality and counting MAY
reuse the private finite String-List runtime fragment. Their nonrecursive
control flow SHALL remain correct at O0. Private definitions, calls, returns,
and containing private aggregates SHALL use exact pointer-bearing prototypes
with physical AMD64 placement selected by LLVM. DWARF and the bundled GDB
renderer SHALL preserve and safely render the source `List Character` identity,
complete multi-scalar characters, and String delimiters.

This rule SHALL add no code-point Character representation, copied String,
implicit normalization, host text API, runtime type tag, type-erased generic
List, public, foreign, serialized, or compiled-library node ABI, foreign
allocator, C/C++ runtime, other-language standard library, or native-ABI
revision. Projections, insertion, concatenation, reversal, removal, range
selection, traversal, higher-order transforms, reclamation beyond process
lifetime, and other element classifiers remain governed by separate rules.
Future compiled-library metadata SHALL encode the Character classifier and
constraint evidence, representation and ownership, lifetime, effects, and
versioned target adapters independently of node offsets, private helper names,
LLVM types/symbols, debug shadows, and physical placement.

### TOPAL-COMPILER-LIST-NAT-CORE-001 — Ordinary immutable Nat Lists

The compiler SHALL admit contextual `Empty` and `Entry` construction for
`List Nat`, immutable binding, ordinary private parameters and results,
package fields, recursively admitted Tuple/Record fields, structural equality
and inequality, complete `Empty`/`Entry (first, rest)` decisions, entry count,
emptiness, canonical display, and debugging. Construction and calls SHALL
evaluate their operands once in source order. Finite entries SHALL be validated
as nonnegative and retain reusable Nat evidence; `+Infinity` SHALL remain a
valid Nat entry. Equality SHALL compare length and corresponding exact numeric
values in order, stopping at the first mismatch without mutating either input.

On Linux x86-64, `Empty` MAY remain a null private pointer and each `Entry` MAY
use an immutable 16-byte node containing the existing exact Int-compatible
pointer and the remaining-node pointer. Equality and counting MAY reuse the
private finite Int-List runtime fragment, including its canonical handling of
the positive-infinity sentinel. Their nonrecursive control flow SHALL remain
correct at O0. Private definitions, calls, returns, and containing private
aggregates SHALL use exact pointer-bearing prototypes with physical AMD64
placement selected by LLVM. DWARF and the bundled GDB renderer SHALL preserve
and safely render the source `List Nat` identity, arbitrary-precision finite
entries, and `+Infinity`.

This rule SHALL add no machine-unsigned width, truncation, wrapping, second
numeric representation, runtime type tag, type-erased generic List, public,
foreign, serialized, or compiled-library node ABI, foreign allocator, C/C++
runtime, other-language standard library, or native-ABI revision. Projections,
insertion, concatenation, reversal, removal, range selection, traversal,
higher-order transforms, reclamation beyond process lifetime, and other
element classifiers remain governed by separate rules. Future compiled-library
metadata SHALL encode Nat constraint evidence and infinity capability,
representation and ownership, lifetime, effects, and versioned target adapters
independently of node offsets, private helper names, LLVM types/symbols, debug
shadows, and physical placement.

### TOPAL-COMPILER-LIST-RATIONAL-CORE-001 — Ordinary immutable Rational Lists

The compiler SHALL admit contextual `Empty` and `Entry` construction for
`List Rational`, immutable binding, ordinary private parameters/results,
package and recursively admitted Tuple/Record fields, structural equality and
inequality, complete decisions, entry count, emptiness, canonical display, and
debugging. Operands SHALL evaluate once in source order. Entries SHALL retain
exact reduced Rational values, including either infinity. Equality SHALL compare
length and corresponding values through canonical Rational comparison.

On Linux x86-64, `Entry` MAY use an immutable 16-byte node containing the
existing Rational descriptor pointer and remaining pointer; `Empty` MAY be
null. A conditional finite nonrecursive fragment MAY implement equality and
counting and SHALL remain correct at O0. LLVM SHALL select physical AMD64
placement for exact private pointer-bearing prototypes. DWARF/GDB SHALL preserve
the `List Rational` identity, exact fractions, and infinities.

This rule SHALL add no floating-point conversion, host numeric API, runtime
type tag, generic/public/foreign/library node ABI, foreign allocator, C/C++
runtime, other-language standard library, or native-ABI revision. Other List
operations, reclamation, and element classifiers remain separately governed.
Future library metadata SHALL encode Rational/node representation, ownership,
lifetime, effects, infinity capability, and versioned target adapters
independently of private offsets, names, LLVM types/symbols, and placement.

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
layout, or native ABI revision. Except for the ordinary boundaries later
admitted by `TOPAL-COMPILER-LIST-INT-PAIR-CORE-001`, other product shapes and
classifiers, pair-returning transformations, and remaining operations over pair
Lists SHALL remain rejected pending later increments.

### TOPAL-COMPILER-LIST-INT-PAIR-CORE-001 — Ordinary Int-pair Lists

The compiler SHALL admit empty/nonempty `List (Int, Int)` construction,
immutable binding, private parameter/result/package and nested Tuple/Record
passage, derived structural equality, complete decisions, count, emptiness,
display, and debugging. List equality SHALL compare both arbitrary-precision
Int fields in source order, stop at the first mismatch, distinguish unequal
lengths, and remain correct at O0.

On Linux x86-64, an Entry MAY retain the existing immutable 24-byte inline node
containing two canonical Int pointers and the remaining pointer. LLVM SHALL own
physical AMD64 placement. This rule SHALL add no allocated Tuple payload,
type-erased container, public/foreign/library ABI, foreign allocator, C/C++
runtime, other-language standard library, or native-ABI revision. Future
metadata SHALL encode the recursive List and positional Tuple classifiers,
exact field types, representation/ownership/lifetime/effects, and target
adapters independently of private offsets, LLVM types, helpers, and debug
shadows.

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
relocation; or native ABI revision. Except for the ordinary inner boundaries
later admitted by `TOPAL-COMPILER-LIST-INT-STRING-PAIR-CORE-001`, deeper
recursion, outer `rest`/`uncons`, other product shapes and element classifiers,
remaining List algorithms, reclamation beyond process lifetime, and versioned
library metadata/adapters remain deferred.

### TOPAL-COMPILER-LIST-INT-STRING-PAIR-CORE-001 — Ordinary Int/String-pair Lists

The compiler SHALL admit empty/nonempty `List (Int, String)` construction,
immutable binding, private parameter/result/package and nested Tuple/Record
passage, derived structural equality, complete decisions, count, emptiness,
display, and debugging. Equality SHALL compare each arbitrary-precision Int and
exact String field in source order, stop at the first mismatch, distinguish
unequal lengths, and remain correct at O0. The same exact inner equality loop
MAY serve the recursive outer-List specialization.

On Linux x86-64, an Entry MAY retain the existing immutable 24-byte inline node
containing the canonical Int pointer, immutable String pointer, and remaining
pointer. LLVM SHALL own physical AMD64 placement. This rule SHALL add no
allocated Tuple payload, type-erased container, public/foreign/library ABI,
foreign allocator, C/C++ runtime, other-language standard library, or native-
ABI revision. Future metadata SHALL encode the recursive List and positional
Tuple classifiers, exact field types, representation/ownership/lifetime/
effects, and target adapters independently of private offsets, LLVM types,
helpers, and debug shadows.

### TOPAL-COMPILER-LIST-STRING-INT-PAIR-CORE-001 — Ordinary String/Int-pair Lists

The compiler SHALL admit empty/nonempty `List (String, Int)` construction,
immutable binding, private parameter/result/package and nested Tuple/Record
passage, derived structural equality, complete decisions, count, emptiness,
display, and debugging. Equality SHALL compare each exact String and arbitrary-
precision Int field in source order, stop at the first mismatch, distinguish
unequal lengths, and remain correct at O0.

On Linux x86-64, an Entry MAY retain the existing immutable 24-byte inline node
used as exact Map-collection input, containing the immutable String pointer,
canonical Int pointer, and remaining pointer. LLVM SHALL own physical AMD64
placement. This rule SHALL add no allocated Tuple payload, type-erased
container, public/foreign/library ABI, foreign allocator, C/C++ runtime, other-
language standard library, Map ABI change, or native-ABI revision. Future
metadata SHALL encode the recursive List and positional Tuple classifiers,
exact field types, representation/ownership/lifetime/effects, and target
adapters independently of private offsets, LLVM types, helpers, and debug
shadows.

### TOPAL-COMPILER-LIST-STRING-PAIR-CORE-001 — Ordinary String-pair Lists

The compiler SHALL admit empty/nonempty `List (String, String)` construction,
immutable binding, private parameter/result/package and nested Tuple/Record
passage, derived structural equality, complete decisions, count, emptiness,
display, and debugging. Equality SHALL compare both exact String fields in
source order, stop at the first mismatch, distinguish unequal lengths, and
remain correct at O0.

On Linux x86-64, an Entry MAY use an immutable 24-byte inline node containing
the two immutable String pointers and remaining pointer. LLVM SHALL own physical
AMD64 placement. This rule SHALL add no allocated Tuple payload, type-erased
container, public/foreign/library ABI, foreign allocator, C/C++ runtime, other-
language standard library, or native-ABI revision. Future metadata SHALL encode
the recursive List and positional Tuple classifiers, exact field types,
representation/ownership/lifetime/effects, and target adapters independently of
private offsets, LLVM types, helpers, and debug shadows.

### TOPAL-COMPILER-LIST-SEQUENCE-001 — Closed ordered List sequences

The compiler SHALL admit the complete shared
`list-sequence-operations.t` regression. For exact finite `List Int` sources it
SHALL implement single-entry and bulk `insert-at`, `split-at`, `take`, `drop`,
indexed `remove`, range and predicate `remove-indexes`, predicate
`remove-values`, all three zip policies, `unzip`, ordered `foreach`, `entries`,
and List identity collection. It SHALL also construct `List String` and collect
its entries into one String in traversal order. Every operation SHALL preserve
the source Lists, evaluate operands once in source order, visit entries in List
order, and produce the same canonical value as the interpreter.

The checked representation SHALL preserve exact source counts for admitted
closed List bindings. It SHALL diagnose a closed boundary, index, or range that
violates `TOPAL-LIST-BOUNDARY-CHECK-001` with
`E-LIST-BOUNDARY-OUT-OF-RANGE` before LLVM generation. Until general dependent
count evidence is implemented, a position not known exactly SHALL be rejected
rather than clamped or compiled with unsound assumptions. `zip-exact` SHALL
remain a generated Result operation: unequal runtime counts SHALL produce
`out-of-range` in lexical domain `root.zip-exact(List,List)`. Predicate removal
and foreach SHALL execute their checked bodies once for each visited entry or
zero-based index and SHALL retain source order at O0.

On Linux x86-64, Int and String List nodes SHALL each contain payload and
remaining pointers; Int-pair nodes SHALL contain both payload pointers and the
remaining pointer; indexed-entry nodes SHALL contain the two Int pointers,
canonical field-order evidence, and the remaining pointer. These layouts and
their alignment SHALL remain private and SHALL be described to LLVM and DWARF
from the checked target layout. Direct generated loops and module-private
helpers SHALL copy only required prefixes, share immutable suffixes where
valid, allocate complete result nodes through the Topal Linux mapping boundary,
and concatenate String data through the Topal String runtime. Correctness SHALL
NOT depend on inlining, folding, vectorization, dead-code elimination, a host
container, callback, indirect call, or foreign runtime.

DWARF and the bundled GDB renderer SHALL expose the semantic Int, String,
pair, and indexed-entry List classifiers and complete bounded values. This rule
SHALL add no undefined symbol, needed library, dynamic relocation, foreign
allocator, C/C++ runtime, other-language standard library, public calling
convention, stable List layout, compiled-library ABI, or native ABI revision.
Future compiled-library metadata SHALL identify recursive List and element
types, pair/record shapes, count evidence, operation and predicate semantics,
ordering, fallibility, allocation effects, ownership, native representation,
and target adapters without publishing private node offsets or helper names.

### TOPAL-COMPILER-FUNDAMENTAL-CONTAINERS-001 — Closed fundamental containers

The compiler SHALL admit the complete shared `fundamental-containers.t`
regression. It SHALL construct `Array 3 Int`, `Set Int`, and `Bag Int` from the
exact finite source `List Int`, and `Map (String, Int)` from the exact finite
pair List under the stated `keep-last` policy. The checked operation family
SHALL also support `reject` and `keep-first`. Array SHALL preserve List order
and extent; Set SHALL eliminate equal entries; Bag SHALL retain total count and
positive multiplicity for each distinct entry; Map SHALL resolve equal keys by
the explicit policy. Set, Bag, and Map physical traversal order SHALL NOT create
a source-language ordering guarantee.

The checked model SHALL preserve container kind, element/key/value classifiers,
exact Array extent, and Map collision policy. A duplicate exact String key with
`reject` SHALL diagnose `E-MAP-KEY-COLLISION` before LLVM emission. An admitted
Array index SHALL be a closed nonnegative exact position. Generic `entry-count`
and `empty?`, `array-at?`, `set-contains?`, `bag-multiplicity`, and `map-lookup`
SHALL produce the values required by their container rules. Every source
operand SHALL be evaluated once in source order, and correctness SHALL remain
independent of optimization.

On Linux x86-64, each value SHALL use a module-private pointer-backed header.
Array MAY retain its immutable source List. Set, Bag, and Map construction MAY
mutate fresh unreachable nodes only until the result is complete; no published
node SHALL be mutated. Helpers SHALL call the canonical exact Int comparator,
String equality, Optional constructors, and Topal Linux mapping allocator.
LLVM SHALL determine physical target placement from the qualified triple and
data layout. No representation in this rule SHALL become a public calling
convention or compiled-library ABI.

DWARF and the bundled GDB renderer SHALL expose the semantic container types,
counts, entries, multiplicities, keys, and values through bounded validating
inspection. This rule SHALL add no undefined symbol, needed library, dynamic
relocation, foreign allocator, host container, C/C++ runtime, other-language
standard library, stable private layout, or native ABI revision. Dynamic Array
positions, dynamically discovered `reject` collisions, other element/key/value
classifiers, general function or library boundaries, persistence,
serialization, and reclamation beyond process lifetime remain deferred. Future
library metadata SHALL identify container kind, classifiers, exact extent,
equality/collision policy, ordering, query fallibility, allocation effects,
ownership, native representation, and target adapters without publishing
private offsets or helper symbols.

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

### TOPAL-COMPILER-TASK-DIRECT-001 — Closed direct task transactions

The compiler SHALL admit a root task classifier and definition containing one
private `Nat` state field, `start (Nat) -> Completed`, zero or more ordinary
`MessageContext`-discarding `Nat`-payload Unit event handlers whose sole action
adds the payload to that state, zero or more `MessageContext`-discarding Unit
request handlers returning the current state as `Result (Nat, ())`, and an
optional declared no-op `terminate (String) -> Unit`. Construction SHALL
evaluate the start operand exactly once, initialize the private state, and
produce a fresh instance with a distinct nonzero identity. An event followed
by a request SHALL commit the complete replacement state before the request
observes it.

For this closed single-root subset, the compiler SHALL use a deterministic
immediate FIFO scheduler. Because every admitted handler explicitly discards
`MessageContext`, the checked model SHALL retain each source-ordered operation
and stable transaction identity while generated O0 code MAY erase the
unobservable context value and queue storage. Such erasure is a checked
language lowering, not an optional LLVM optimization. The declared queue bound
and scheduling policy SHALL remain in target-independent task metadata. Task
streams beyond `TOPAL-COMPILER-TASK-STREAM-001`, overlapping delivery,
observable context, queue overflow, and termination delivery SHALL remain
rejected.

Linux x86-64 lowering SHALL allocate a private Topal-owned instance containing
identity, lifecycle state, and the current arbitrary-precision Nat pointer.
State replacement SHALL store a newly computed immutable Nat only after the
addition completes. This private object SHALL NOT define a public, foreign,
persistent, serialized, function, or compiled-library ABI. LLVM SHALL select
all physical calling and data-layout details; no C/C++ runtime or
other-language standard library may be linked.

DWARF SHALL expose the source task classifier, identity, lifecycle state, and
named private state. The bundled GDB renderer SHALL bound its read and validate
identity, lifecycle tag, and state pointer before rendering. Future
compiled-library metadata for tasks SHALL identify the language and metadata
revision, nominal task and definition identities, queue and scheduler policy,
state schema, handler kind and complete signature, context/transaction model,
effects and authorities, ownership and termination contract, debug provenance,
and native-representation adapter independently of LLVM types and symbols.

### TOPAL-COMPILER-TASK-STREAM-001 — Closed one-yield task stream transaction

The compiler SHALL extend `TOPAL-COMPILER-TASK-DIRECT-001` with an ordinary
stream handler whose `MessageContext` and Unit payload are explicitly
discarded, whose yield classifier is `Nat`, whose resumption classifier is
Unit, whose final result is `Result (Unit, ())`, and whose body yields the
current private Nat state exactly once before returning Unit. Applying that
handler SHALL establish one affine stream transaction with a stable identity.
Its yield SHALL observe state committed by every preceding event, and the
stream's successful final result SHALL commit before a following request is
delivered.

The checked model SHALL retain the stream's complete yield, resume, and final
result directions; owning task instance; handler identity; and transaction
identity. Traversal SHALL consume the stream exactly once. For this closed
immediate-FIFO case, the traversal action SHALL be an inert Unit resumption and
the handler performs no post-resumption state access. Generated O0 code MAY
therefore inline the single suspension, erase the unobservable resumption and
continuation carriers, and load private state directly at the source yield.
This is a checked language lowering and SHALL NOT rely on an LLVM optimization.
General stream bodies, observable context or resumption, multiple yields,
post-resumption state access, abandonment, close delivery, overlapping
delivery, and termination interaction SHALL remain rejected.

Linux x86-64 lowering SHALL reuse the private Topal-owned task instance, state
load, and Result-success representation without adding a foreign runtime,
C/C++ standard library, other-language standard library, or public ABI. LLVM
SHALL continue to select physical layout and calling-convention details. DWARF
SHALL expose the nominal owning task, affine Generator directions, stream
binding, source yield, and private state. Future compiled-library metadata
SHALL describe stream handler directions, transaction and suspension identity,
state-authority release/reacquisition, ownership and consumption, close and
termination behavior, effects and authorities, debug provenance, and native
adapter requirements independently of LLVM types, layouts, and symbols.

### TOPAL-COMPILER-EXTERNAL-LOCATION-001 — Closed checked external location

The compiler SHALL admit the unchanged `external-layout-location.t` regression
as one closed external-storage graph. The checked model SHALL retain the exact
identities, semantic classifiers, storage sizes, encoding families, byte order,
access policy, alignment, product packing and fields, tagged-sum tags and
placement, array extent, element layout, and stride of its five layout
declarations. It SHALL also retain the address-range cache policy, minimum
physical access size, medium, inclusive bounds and value identity; the
address-offset range identity, byte alignment and checked value; and the
location subtype, layout identity, range identity, and offset.

Construction SHALL prove nonnegative ordered bounds, positive aligned offset,
range membership, complete layout fit, physical-access compatibility, and exact
32-bit unsigned representability of the stored Nat. The admitted source write
and following read SHALL remain distinct source-ordered operations at O0 and
SHALL produce the same layout-backed Nat snapshot as the interpreter. Unknown,
missing, duplicate, inapplicable, inconsistent, dynamic, unrepresentable, or
unsupported layout fields and values SHALL fail before LLVM lowering. Other
layout families, dynamic construction, general byte encoding/decoding,
uninitialized reads, public/function/persistent/library location boundaries,
and general fallible access SHALL remain rejected.

The source declares an abstract address range but grants no Linux device,
mapping, or native-adapter authority. Generated code therefore SHALL NOT
dereference the numeric MMIO address or silently claim host-device integration.
Linux x86-64 lowering SHALL use a private Topal-owned location header containing
the semantic range start, offset, initialization state, and immutable semantic
snapshot; ordered noinline Topal runtime calls SHALL perform the admitted write
and read. Allocation and termination SHALL use the existing direct Linux
syscalls, with no C/C++ runtime, other-language standard library, foreign
allocator, undefined symbol, or public ABI. Real MMIO requires a future
explicit platform adapter and remains outside this increment.

LLVM SHALL own the private header's physical layout, pointer placement, register
selection, and instruction scheduling. The frontend's schema validation,
authority rejection, source ordering, and representability proof SHALL hold at
O0 and SHALL NOT depend on optional LLVM optimization. DWARF SHALL expose the
nominal layout-backed value and Location header, address evidence,
initialization state, stored snapshot, source bindings, access operations, and
frames. The validating GDB renderer SHALL bound its reads and reject null,
unreadable, noncanonical, or inconsistent private state.

Future compiled-library metadata SHALL encode the language and schema revision,
layout identity and complete field graph, semantic type identity, sizes,
encoding, byte order, access and alignment, range/offset/location identities,
medium and ordering, effects and authorities, fallibility, ownership/lifetime,
platform-adapter requirement, and debug provenance independently of LLVM types,
private headers, target layouts, and symbols.

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

### TOPAL-COMPILER-NATIVE-SERIALIZATION-001 — Closed canonical native streams

For a compiler-created, authority-free value that is statically known as
literal Unit or Boolean, exact Int, known String, or one direct Tuple or Record
whose fields are those supported scalar values, the compiler SHALL admit
`version (lang serialize) value`, the reusable
`lang version (lang serialize)` operation, and `lang deserialize stream`. It SHALL
derive exactly one finite canonical protocol 1.0 stream under
`TOPAL-SER-HEADER-001` through `TOPAL-SER-CANON-001`, using the selected Topal
language revision, target little-endian value encoding, recursive first-use
type-definition order, and source declaration order for Record fields.

At O0, serialization SHALL evaluate the source value exactly once. Linux
x86-64 lowering SHALL place the already validated expected bytes in immutable
executable storage, copy them into private Topal-owned stream storage, and create
a private Topal-owned `{data, byte_count}` descriptor for that copy. The
published copy SHALL thereafter be treated as immutable. Before deserialization
returns the retained once-evaluated value, generated code SHALL compare the
descriptor byte count and every byte in the copy with the canonical immutable
expected stream. A mismatch SHALL terminate through the compiler-runtime
corruption path and SHALL NOT expose the value.

This increment SHALL NOT parse arbitrary runtime streams, admit externally
supplied streams, infer unknown scalar contents, serialize nested or indirect
aggregates or other classifiers, or permit a SerializationStream across a function,
persistent, public, or compiled-library machine boundary. It SHALL NOT link a
foreign serialization implementation, C/C++ runtime, or other-language
standard library, and SHALL NOT depend on an optional LLVM optimization.

DWARF SHALL expose `SerializationStream` as a pointer to an inspectable private
header containing the byte address and count. The bundled GDB renderer SHALL
bound memory reads, validate `TOPALSER` magic, and render the canonical byte
count. Any future compiled-library interface carrying this operation SHALL
identify the protocol revision, language identity and revision, canonical type
identities and schemas, field order, byte-order contract, and authority profile
independently of private LLVM types, descriptors, and native symbols.

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
