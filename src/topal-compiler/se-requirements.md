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

These additions cover the admitted scalar cases of
`TOPAL-FUNCTION-STATIC-NULLARY-001`, `TOPAL-FUNCTION-STATIC-UNARY-001`,
`TOPAL-FUNCTION-STATIC-BINARY-001`, `TOPAL-FUNCTION-BLOCK-001`,
`TOPAL-FUNCTION-ORDINARY-001`, `TOPAL-FUNCTION-CALL-CHAIN-001`,
`TOPAL-FUNCTION-LOCAL-SCOPE-001`, `TOPAL-FUNCTION-OVERLOAD-001`,
and `TOPAL-FUNCTION-FORWARD-DECLARATION-001`. Dynamic structural applicability,
function values, nested functions, recursion and its overload-identity rule,
and other function forms remain in later increment-3 dispositions.

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
Nat, Rational, Comparison, ErrorCode, String, payload-free source Enum,
`Optional Int`, `Optional Rational`, `Optional String`, and another admitted
positional product.
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

## TOPAL-COMP-DEBUG-001 — DWARF and GDB

Debug-enabled O0 output shall map generated source functions, parameters,
immutable scalar locals, lexical scopes, and instructions to Topal files and
source locations, emit DWARF 5 through LLVM, retain frame pointers, provide GDB
renderers for private arbitrary-precision Int, Rational, `Range Int`,
`Range Rational`, `Optional Int`, and `Optional String` objects, describe
source-declared nominal enums with their alternative labels,
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
