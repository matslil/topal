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
compiler increment 2c-b3; Optional Result composition and the remaining String
operations retain their later roadmap dispositions.

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
source/compiled libraries remain rejected. This realizes
`TOPAL-COMPILER-NAMESPACE-DATA-001` for increment 6b2a under
`TOPAL-NAMESPACE-ROOT-001`, `TOPAL-NAMESPACE-ALIAS-001`,
`TOPAL-NAMESPACE-SNAPSHOT-001`, `TOPAL-NAMESPACE-CLASSIFIER-001`, and
`TOPAL-NAMESPACE-ALIAS-CHAIN-001`.

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

The compiler shall reject lexical data captures and anonymous product parameter
patterns until an explicit cross-frame capture representation exists. This
increment shall add no function pointer, indirect call, closure allocation,
closure or Function runtime, foreign dependency, C/C++ runtime, other-language
standard library, public callable ABI, or `topal-native/6` revision. Escaping
closures, Function results and aggregate boundaries, and published callable
interfaces remain rejected. This realizes
`TOPAL-COMPILER-ANONYMOUS-DIRECT-001` and the admitted portion of
`TOPAL-FUNCTION-ANONYMOUS-001`, `TOPAL-SYN-GRAMMAR-001`, and
`TOPAL-TYPE-CALL-001` for compiler increment 3b2-b5m.

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
attribute and no package runtime or allocation. Multiple or mixed packages,
non-scalar/nested fields, opaque or non-prefix/reordered labeled values, and
invocation-dependent defaults remain rejected until their complete evaluation,
storage, and public ABI rules are implemented. This shall add no foreign
dependency, C/C++ runtime, other-language standard library, public aggregate
ABI, or `topal-native/6` revision. This realizes
`TOPAL-COMPILER-PACKAGED-OPERAND-001`, `TOPAL-FUNCTION-PACKAGED-OPERAND-001`,
and `TOPAL-TYPE-CALL-001` for compiler increment 3b2-b5n.

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
order, invalid context use, forwarding rejection, IR, artifact independence,
and debugger observation.

This increment shall add no global context storage, lookup table, closure
allocation/runtime, function pointer, indirect call, foreign dependency, C/C++
runtime, other-language standard library, public closure ABI, or
`topal-native/6` revision. Aggregate, Scope, and Function captures,
cross-function forwarding, anonymous/escaping closures, qualified root data in
functions, and public/library contexts remain rejected. This realizes
`TOPAL-COMPILER-CONTEXT-CAPTURE-001` and `TOPAL-CONTEXT-SELECT-001` for compiler
increment 6c1.

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
parameters and results, and source display. This scalar rule does not admit
Effect Lists or aggregate results; `TOPAL-COMP-TUPLE-RESULT-001` separately
admits a qualified Tuple result containing this value. Effect Lists remain
unsupported until their general representation is admitted.

The backend shall use a sealed zero-data scalar only in Topal-private signatures,
emit a distinct `Effect` DWARF enumeration for GDB, and require neither
allocation nor an effect-specific runtime function. Canonical display shall use
the existing Topal-owned Linux write boundary. No C/C++ runtime, other-language
standard library, public integer ABI, or `topal-native/6` revision is permitted.
This realizes `TOPAL-COMPILER-EFFECT-EMPTY-001` for compiler increment 7a.

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
a C/C++ runtime, or introduce load-time pointer relocations. Unbounded forms,
infinite endpoints, and collection selection remain outside this increment.

This requirement covers `TOPAL-RANGE-BOUNDS-001`,
`TOPAL-RANGE-MEMBERSHIP-001`, `TOPAL-RANGE-RATIONAL-001`,
`TOPAL-RANGE-CLASSIFIER-001`, `TOPAL-RANGE-INTERSECTION-001`,
`TOPAL-RANGE-EMPTY-001`, and `TOPAL-RANGE-BOUND-001`. It realizes
`TOPAL-COMPILER-RANGE-001` for compiler increment 2d-a.

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

Constraint application/evidence, capturing predicates, function or aggregate
boundaries, and public compiled-library constraint identities remain rejected.
This shall add no allocation, constraint runtime, foreign dependency, C/C++
runtime, other-language standard library, or `topal-native/6` revision. This
realizes `TOPAL-COMPILER-CONSTRAINT-VALUE-001`,
`TOPAL-ABSTRACTION-CONSTRAINT-CLASSIFIER-001`, and the construction-only portion
of `TOPAL-TYPE-CONSTRAINT-001` for compiler increment 8a1.

## TOPAL-COMP-DEBUG-001 — DWARF and GDB

Debug-enabled O0 output shall map generated source functions, parameters,
immutable scalar locals, lexical scopes, and instructions to Topal files and
source locations, emit DWARF 5 through LLVM, retain frame pointers, provide GDB
renderers for private arbitrary-precision Int, Rational, `Range Int`,
`Range Rational`, `Optional Int`, and `Optional String` objects, describe
source-declared nominal enums and retained Constraint identities with their
alternative labels,
distinguish selected overload and static-function frames, and pass automated
GDB breakpoint, value, and backtrace scenarios.

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
