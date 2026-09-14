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
