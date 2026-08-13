# Interpreter requirements

These tool requirements refine `TOPAL-REQ-TOOLS-001`,
`TOPAL-REQ-INTEROP-001`, and `TOPAL-REQ-TRACE-001` for the `topal`
interpreter. The three modes and their command-line selection record the
implementation intent approved for the initial interpreter work.

## TOPAL-INTP-MODE-001 — Script mode

The interpreter shall use script mode by default. It shall read one named
source file, or standard input when no file is named, execute it, write the
final value to standard output, and report diagnostics on standard error with
a nonzero status. A first `#!` line shall be treated as an operating-system
launcher directive rather than Topal source.

## TOPAL-INTP-MODE-002 — Interactive mode

`--interactive` shall start a persistent evaluation session which reads source
from standard input, prints each successful value, reports a failed input
without ending the session, and presents a prompt when standard input is a
terminal.

## TOPAL-INTP-MODE-003 — Conformance-test mode

`--test` shall execute scripted input with the same language result and
diagnostic behavior as script mode. It shall additionally write semantic
decision events to standard error as JSON Lines using the versioned
`topal.test-trace/1` envelope. Each event shall identify its stable event name,
the governing specification rule, and deterministic decision detail suitable
for comparison with a future compiler trace.

Trace collection shall not change the program result, accepted language, or
decision order. Tests shall compare semantic event fields rather than runtime
addresses, elapsed time, or implementation-specific debug output.

## TOPAL-INTP-SUBSET-001 — Explicit revision subset

The interpreter shall reject valid `design-0` syntax which it does not yet
implement with an explicit unsupported-syntax diagnostic. It shall not guess
semantics for syntax absent from the formal language revision.

## TOPAL-INTP-SUBSET-002 — Immutable bindings

The implemented subset shall execute source-ordered `is` bindings and name
lookups according to `TOPAL-SYN-BIND-001`. A binding initializer shall complete
before the name becomes visible, rebinding in one session scope shall be
rejected, and a non-final value expression shall be rejected rather than
silently discarded. A successful declaration statement produces `Unit`.

## TOPAL-INTP-SUBSET-003 — Exact integer literal bases

The implemented subset shall accept arbitrary-precision decimal, binary,
octal, and hexadecimal integer literals with the grouping validation required
by `TOPAL-SYN-NUM-001`. Display shall use the value's canonical decimal form;
lexical radix and grouping shall not change numeric identity.

## TOPAL-INTP-SUBSET-004 — Finite exact integer addition

The interpreter shall consume the shared source and syntax layers and evaluate
left-associated finite `Int` addition according to `TOPAL-NUM-ADD-001` in all
three modes. Test mode shall distinguish callable selection from exact result
construction using stable semantic events and governing rule identities.

## TOPAL-INTP-SUBSET-005 — Finite exact integer negation and subtraction

The interpreter shall evaluate spaced prefix negation and binary subtraction
according to `TOPAL-NUM-NEG-001` and `TOPAL-NUM-SUB-001`. Adjacent signed
literals shall retain their distinct literal-construction trace path. Binary
subtraction shall compose left-to-right with every ordinary application.

## TOPAL-INTP-SUBSET-006 — Finite exact integer multiplication

The interpreter shall evaluate finite `Int` multiplication according to
`TOPAL-NUM-MUL-001` without overflow. Multiplication shall use ordinary
left-to-right application order rather than conventional hidden precedence,
and test mode shall expose selection and exact construction decisions.

## TOPAL-INTP-SUBSET-007 — Finite exact integer division

The interpreter shall divide finite `Int` values into canonical exact
`Rational` values according to `TOPAL-NUM-RATIONAL-001` and
`TOPAL-NUM-DIV-001`. It shall display the result as
`Rational ( numerator, denominator )`, including a denominator of one, so the
observable result retains its type. A statically evident zero divisor shall be
rejected according to `TOPAL-NUM-DIVZERO-001`.

## TOPAL-INTP-SUBSET-008 — Division evidence traces

Test mode shall record whether the exact-division nonzero obligation was proved
or refuted before overload selection. A refuted obligation shall identify
`TOPAL-NUM-DIVZERO-001`, suppress selection and evaluation events for that
application, and place the diagnostic at the divisor source range.

## TOPAL-INTP-SUBSET-009 — Finite natural integer exponentiation

The interpreter shall evaluate `Int ^ Nat -> Int` according to
`TOPAL-NUM-POW-001`, including `0 ^ 0 = 1`, without restricting exponent or
result to a machine width. It shall prove the exponent's finite nonnegative
classification before overload selection, reject a negative exponent at its
source range, and retain ordinary left-to-right application order.

## TOPAL-INTP-SUBSET-010 — Exact rational literals

The interpreter shall construct fractional decimal and base-ten exponent
literals as canonical exact `Rational` values under
`TOPAL-NUM-RATIONAL-LITERAL-001`. All modes shall preserve arbitrary exponent
size, grouping validity, sign, and the visible rational result type.

## TOPAL-INTP-SUBSET-011 — Finite exact rational arithmetic

The interpreter shall evaluate same-domain finite `Rational` negation,
addition, subtraction, multiplication, and division according to
`TOPAL-NUM-RAT-NEG-001` through `TOPAL-NUM-RAT-DIV-001`. Every result shall be
canonical, rational zero division shall use the existing nonzero evidence
contract, and no implicit mixed-domain conversion shall be invented.

## TOPAL-INTP-SUBSET-012 — Mixed finite exact arithmetic

The interpreter shall apply the canonical `Int -> Rational` conversion from
`TOPAL-NUM-INT-RATIONAL-CONVERT-001` when it makes an existing rational
arithmetic overload applicable. It shall not synthesize mixed overloads, apply
the conversion to the `Int ^ Nat` exponent position, or perform the reverse
conversion without evidence. Test traces shall record each conversion before
overload selection.

## TOPAL-INTP-SUBSET-013 — Literal-preserving strings

The interpreter shall construct ordinary and exactly tagged string literals
according to `TOPAL-SYN-STRING-001`, preserving Unicode contents, newlines,
backslashes, and braces without escapes or interpolation. Display shall emit a
deterministic valid source-like literal, selecting and extending the tag `text`
when the value contains quotes. Unterminated input shall remain available to
shared syntax consumers and be rejected for execution.

## TOPAL-INTP-SUBSET-014 — Pinned Unicode source semantics

The interpreter and its shared frontend shall use the exact Unicode 17.0.0
normalization and identifier-property data required by
`TOPAL-SYN-UNICODE-001`. They shall reject non-NFC identifiers and literal tags
without normalizing source spellings, preserve string contents, and expose the
language-context Unicode version through deterministic version output and test
trace context selection. A dependency with different Unicode tables shall fail
the build rather than silently change source acceptance.

## TOPAL-INTP-SUBSET-015 — Unit product

All interpreter modes shall parse and evaluate the zero-field product `()` as
the sole `Unit` value according to `TOPAL-SYN-GRAMMAR-001` and
`TOPAL-TYPE-PRODUCT-001`. Test mode shall identify construction of `Tuple()` in
a stable decision event.

## TOPAL-INTP-SUBSET-016 — Positional products

All interpreter modes shall evaluate single-line positional products according
to `TOPAL-SYN-GRAMMAR-001` and `TOPAL-TYPE-PRODUCT-001`. Parentheses without a
comma remain grouping; a trailing comma distinguishes a one-field `Tuple`.
Fields evaluate left-to-right, display preserves product arity, and test mode
records the constructed field count.

## TOPAL-INTP-SUBSET-017 — Delimiter-aware continuation

The shared parser shall ignore statement-separating newlines inside paired
parentheses as required by `TOPAL-SYN-INDENT-001` and
`TOPAL-SYN-GRAMMAR-001`. Script and test modes shall evaluate multiline grouped
and product expressions. Interactive mode shall retain incomplete parenthesized
input and continue reading until the closing delimiter arrives.

## TOPAL-INTP-SUBSET-018 — Boolean literals

All interpreter modes shall evaluate reserved `true` and `false` literals as
the two `Boolean` values according to `TOPAL-TYPE-BOOLEAN-001`. The spellings
shall not participate in binding or name resolution, shall not convert to
numbers, and test mode shall trace literal construction.

## TOPAL-INTP-SUBSET-019 — Fundamental equality

All interpreter modes shall evaluate `=` according to
`TOPAL-TYPE-EQUALITY-001` for `Unit`, `Boolean`, exact numbers, strings, and
positional products. Mixed `Int` and `Rational` operands shall use the canonical
conversion and trace it before equality overload selection. Operands without
shared equality evidence shall report no applicable overload rather than
returning `false`; test mode shall trace the selected operation and result.

## TOPAL-INTP-SUBSET-020 — Derived inequality

All interpreter modes shall evaluate `!=` as the Boolean negation of the
applicable `TOPAL-TYPE-EQUALITY-001` result. It shall preserve equality's
conversion, evidence, tuple, and error behavior, and shall appear as one
longest-match callable symbol in shared syntax and test traces.

## TOPAL-INTP-DIAG-001 — Actionable source diagnostics

Interpreter diagnostics shall retain stable machine-readable codes while their
human rendering identifies severity, source name, line and column, the relevant
source line, a Unicode-column-aligned marker, and actionable help when a safe
general correction exists. Script filenames, standard input, and interactive
input shall have deterministic source labels. Rendering concerns shall not
change interpreter semantics or test trace events. When an unbound name has one
deterministically closest visible binding within a conservative edit-distance
threshold, help shall suggest that spelling without changing name resolution.

## TOPAL-INTP-SUBSET-021 — Exact ordering predicates

All interpreter modes shall evaluate `<`, `>`, `<=`, and `>=` for finite `Int`
and `Rational` operands according to `TOPAL-NUM-COMPARE-001`. Mixed operands
shall use and trace canonical exact conversion before overload selection. Test
mode shall record the three-way comparison decision independently from the
derived Boolean predicate result. Other value domains shall report no
applicable overload.

## TOPAL-INTP-SUBSET-022 — Lexicographic tuple ordering

All interpreter modes shall derive lexicographic ordering for equal-arity
positional products whose corresponding fields provide implemented
`TotalOrder`, according to `TOPAL-TYPE-ORDERING-001`. Comparison shall stop at
the first non-equal field, apply canonical numeric field conversions when
needed, reject different arities or unsupported fields, and trace the resulting
tuple-ordering decision separately from numeric comparison.

## TOPAL-INTP-EXAMPLE-001 — Executable feature examples

Every implemented language-feature increment shall add or extend a related
runnable source file under `examples/interpreter/`. The interpreter functional
suite shall execute every such source file successfully in default script mode,
including its hashbang when present, so examples cannot silently drift from the
implemented language.

## TOPAL-INTP-SUBSET-023 — Discard declarations

All interpreter modes shall evaluate `_ is expression` according to
`TOPAL-SYN-BIND-001`, including every semantic decision and diagnostic produced
by the initializer, then produce `Unit` without introducing a binding. The
complete `_` spelling shall be reserved from identifier lookup, and test mode
shall record the discard decision.

## TOPAL-INTP-SUBSET-024 — Labeled record products

All interpreter modes shall evaluate products whose fields are all labeled as
anonymous `Record` values according to `TOPAL-SYN-GRAMMAR-001` and
`TOPAL-TYPE-PRODUCT-001`. Labels shall be unique, field values shall evaluate
left-to-right, display shall preserve source field order, and test mode shall
record record construction. Products mixing labeled and positional fields
shall be rejected as invalid syntax with help suggesting explicit nesting.

## TOPAL-INTP-SUBSET-025 — Record field selection

All interpreter modes shall evaluate `record label` as total static field
selection according to `TOPAL-TYPE-PRODUCT-001`. Selection shall group with its
record before later ordinary application, return the field's exact value, and
diagnose an absent label at the label source range. Test mode shall record the
selected label without evaluating it as a name.

## TOPAL-INTP-SUBSET-026 — Plain string concatenation

All interpreter modes shall evaluate `left concat right` for two plain
`String` values according to `TOPAL-STRING-CONCAT-001`, preserving their exact
Unicode sequences without normalization or separators. Test mode shall expose
overload selection and exact concatenation as separate semantic decisions.

Adjacent string literals shall additionally compose according to
`TOPAL-STRING-LITERAL-COMPOSE-001`. This implicit construction shall not extend
to bindings, function results, or other runtime string expressions.

## TOPAL-INTP-SUBSET-027 — Empty string construction

All interpreter modes shall evaluate `empty String` according to
`TOPAL-STRING-EMPTY-001` as the unique zero-scalar plain string. Test mode shall
record callable selection and construction independently.

## TOPAL-INTP-SUBSET-028 — String character count

All interpreter modes shall evaluate `character-count text` for a plain
`String` according to `TOPAL-STRING-CHARACTER-COUNT-001`, using the exact
Unicode segmentation data selected by the language context. The result shall
be a finite nonnegative `Int`. Test mode shall record callable selection and
the segmentation result independently.

## TOPAL-INTP-SUBSET-029 — Derived record equality

All interpreter modes shall derive `=` and `!=` for anonymous records with the
same labeled field set when every corresponding field supports implemented
equality, according to `TOPAL-TYPE-EQUALITY-001`. Construction order shall not
affect applicability or the result. Field comparison shall retain existing exact
numeric conversions. Different record shapes or unsupported fields shall report
no applicable equality overload.

## TOPAL-INTP-SUBSET-030 — String sequence entry count

All interpreter modes shall evaluate `entry-count text` for a plain `String`
according to `TOPAL-STRING-ENTRY-COUNT-001`, producing the same finite
nonnegative `Int` as `character-count text`. Test mode shall identify selection
of the generic sequence operation separately from its character segmentation
result.

## TOPAL-INTP-SUBSET-031 — Prospective UTF-8 String byte count

All interpreter modes shall evaluate `text byte-count Utf8` for a plain
`String` according to `TOPAL-STRING-UTF8-BYTE-COUNT-001`, producing a finite
nonnegative `Int` without changing the String value or normalizing its preserved
sequence. Test mode shall record operation selection and the exact byte count
as separate decisions. Other encodings remain explicitly unsupported.

## TOPAL-INTP-SUBSET-033 — Explicit String NFC normalization

All interpreter modes shall evaluate `text normalize NFC` for a plain `String`
according to `TOPAL-STRING-NORMALIZE-NFC-001`, using the exact Unicode tables
selected by the language context. The operation shall not affect the source
binding or introduce implicit normalization elsewhere. Test mode shall record
operation selection and whether normalization changed the preserved sequence.
## TOPAL-INTP-SUBSET-032 — String emptiness predicate

The shared frontend shall accept a single terminal `?` as part of a predicate
identifier according to `TOPAL-SYN-LEX-001`. All interpreter modes shall
evaluate `empty? text` for a plain `String` according to
`TOPAL-STRING-EMPTY-PREDICATE-001`. Test mode shall record predicate selection
and its Boolean result independently.

## TOPAL-INTP-SUBSET-034 — Static nullary functions

All interpreter modes shall declare and call zero-parameter static functions
with one indented expression body according to
`TOPAL-FUNCTION-STATIC-NULLARY-001`. Function bodies shall remain unevaluated at
declaration, capture only earlier visible bindings, persist in interactive
sessions, validate their explicit result classifier, and execute through shared
debugger checkpoints. Test mode shall record declaration, selection, entry,
body decisions, and return in order.

## TOPAL-INTP-SUBSET-035 — Explicit String NFD normalization

All interpreter modes shall evaluate `text normalize NFD` for a plain `String`
according to `TOPAL-STRING-NORMALIZE-NFD-001`, using the exact Unicode tables
selected by the language context. The operation shall preserve the input
binding and remain explicit. Test mode shall record operation selection and
whether normalization changed the preserved sequence.

## TOPAL-INTP-SUBSET-036 — Static unary functions

All interpreter modes shall declare and call one-parameter static functions
according to `TOPAL-FUNCTION-STATIC-UNARY-001`. Calls shall evaluate and
validate the argument in caller scope, bind the parameter only within the
captured lexical function scope, validate the result classifier, and expose
argument binding plus nested body checkpoints to test traces and the debugger.

## TOPAL-INTP-SUBSET-037 — Static binary infix functions

All interpreter modes shall declare and call static functions with two typed
operands according to `TOPAL-FUNCTION-STATIC-BINARY-001`. Calls shall use infix
application, diagnose classifier mismatches before function entry, bind
validated operands in declaration order, and expose both bindings and nested
body checkpoints to test traces and the debugger.

## TOPAL-INTP-SUBSET-038 — Multi-statement function bodies

All interpreter modes shall execute one-or-more-statement function bodies
according to `TOPAL-FUNCTION-BLOCK-001`, including invocation-local immutable
bindings and final-result validation. Test traces and reversible debugger
history shall preserve the ordered statement decisions between function entry
and return.

## TOPAL-INTP-SUBSET-039 — Explicit function return

All interpreter modes shall execute `return expression` according to
`TOPAL-FUNCTION-RETURN-001`, enforce the nearest function boundary, validate the
returned value, and skip later body statements. Test traces and reversible
debugger history shall expose the explicit-return decision and omit skipped
decisions.

## TOPAL-INTP-SUBSET-040 — Ordinary runtime functions

All interpreter modes shall declare and call ordinary `fn` functions according
to `TOPAL-FUNCTION-ORDINARY-001`, supporting the same implemented parameter and
body subsets as static functions while omitting the static-evaluation
guarantee. Test traces and reversible debugger history shall distinguish
ordinary function decisions from static ones.

## TOPAL-INTP-SUBSET-041 — Nested function call chains

All interpreter modes shall execute acyclic function-to-function calls according
to `TOPAL-FUNCTION-CALL-CHAIN-001`, enforce static-to-static dependencies, and
preserve fresh invocation scopes. Test traces and reversible debugger history
shall expose nested callee entry, decisions, and return before caller return.

## TOPAL-INTP-SUBSET-042 — Function-local lexical shadowing

All interpreter modes shall implement invocation-local shadowing according to
`TOPAL-FUNCTION-LOCAL-SCOPE-001`, distinguishing same-scope duplicates from
legal shadowing of captured bindings and restoring outer visibility after
return. Test traces and reversible debugger history shall preserve both the
local decisions and the unchanged outer state.

## TOPAL-INTP-SUBSET-043 — Ordered typed function overloads

All interpreter modes shall retain same-name function overloads and select them
according to `TOPAL-FUNCTION-OVERLOAD-001`, evaluating the argument once and
rejecting duplicate input/staticness signatures. Diagnostics shall report
available signatures when none applies. Test traces and reversible debugger
history shall identify the selected signature before entry.

## TOPAL-INTP-SUBSET-044 — Complete Boolean decision tables

All interpreter modes shall execute complete Boolean decision tables inside
implemented function bodies according to `TOPAL-DECISION-BOOLEAN-001`, delaying
unselected actions and preserving source rule order. Interactive mode shall
retain the declaration until its deeper-indented rules are complete. Test
traces and reversible debugger history shall expose consideration and selection.

## TOPAL-INTP-SUBSET-045 — Comparison decision matchers

All interpreter modes shall execute comparison decision matchers according to
`TOPAL-DECISION-COMPARISON-001`, evaluating the subject once, applying each
comparison in source order, and delaying unselected actions. Test traces and
reversible debugger history shall expose comparison and selection reasons.

## TOPAL-INTP-SUBSET-046 — Proven decreasing Int recursion

All interpreter modes shall execute self-recursive unary `Int` functions only
after proving the structural decrease required by
`TOPAL-FUNCTION-RECURSION-INT-001`. Unproven cycles shall remain diagnostics.
Test traces and reversible debugger history shall expose proof acceptance and
each nested recursive descent.

## TOPAL-INTP-SUBSET-047 — Proven increasing Int recursion

All interpreter modes shall execute unary `Int` recursion which increases
toward a guarded upper bound only after proving
`TOPAL-FUNCTION-RECURSION-INT-INCREASING-001`. Test traces and reversible
debugger history shall expose the distinct proof rule on declaration and every
recursive descent.

## TOPAL-INTP-SUBSET-048 — Comparison matcher operand expressions

All interpreter modes shall parse and evaluate complete comparison matcher
operand expressions according to `TOPAL-DECISION-OPERAND-EXPRESSION-001`, using
the same application and grouping semantics as ordinary expressions. Test
traces and reversible debugger history shall retain operand decisions before
the containing matcher selection.

## TOPAL-INTP-SUBSET-049 — Nested lexical functions

All interpreter modes shall declare and call functions inside function bodies
according to `TOPAL-FUNCTION-NESTED-001`, capturing invocation-local bindings
without leaking the nested name. Test traces and reversible debugger history
shall expose nested declaration and call decisions within the outer frame.

## TOPAL-INTP-SUBSET-050 — Exhaustive Boolean decision tables

All interpreter modes shall execute a Boolean decision table containing both
literal matchers without requiring `otherwise`, according to
`TOPAL-DECISION-BOOLEAN-001`. Test traces and reversible debugger history shall
identify which exhaustive literal rule was considered and selected.

## TOPAL-INTP-SUBSET-051 — Forward function declarations

All interpreter modes shall allow an earlier function body to call a later
function declaration with a complete explicit header according to
`TOPAL-FUNCTION-FORWARD-DECLARATION-001`. Test traces and reversible debugger
history shall retain the nested selection and entry in execution order.

## TOPAL-INTP-SUBSET-052 — Proven mutual decreasing Int recursion

All interpreter modes shall execute a closed mutual recursion cycle only after
proving every participating edge according to
`TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001`. An isolated candidate or a cycle with
one invalid edge shall remain rejected. Test traces and reversible debugger
history shall expose candidate edges, completed cycle proof, and descent.

## TOPAL-INTP-SUBSET-053 — Proven mutual increasing Int recursion

All interpreter modes shall execute a closed mutually increasing `Int` cycle
only after proving every edge according to
`TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001`. A mixed-direction cycle
shall remain rejected. Test traces and reversible debugger history shall expose
the increasing proof reason separately from decreasing mutual recursion.

## TOPAL-INTP-SUBSET-054 — Overload-specific recursion identity

All interpreter modes shall treat each selected input signature as a distinct
recursion identity according to `TOPAL-FUNCTION-RECURSION-OVERLOAD-IDENTITY-001`.
A call between same-named distinct overloads shall execute without recursion
proof, while a return to the active signature shall retain the proof requirement.
Test traces and reversible debugger history shall expose both selections.

## TOPAL-INTP-SUBSET-055 — Positive literal recursion steps

All interpreter modes shall prove direct and mutual bounded `Int` recursion
using any positive literal step according to
`TOPAL-FUNCTION-RECURSION-INT-POSITIVE-STEP-001`, while rejecting zero,
negative, runtime, and wrong-direction steps. Test traces and reversible
debugger history shall retain the applicable direction-specific proof rule.

## TOPAL-INTP-SUBSET-056 — Multiple recursive calls

All interpreter modes shall execute an action containing multiple recursive
calls only when every call is proven according to
`TOPAL-FUNCTION-RECURSION-ALL-CALLS-001`. One invalid branch shall reject the
function's recursive execution. Test traces and reversible debugger history
shall expose each valid descent in evaluation order.

## TOPAL-INTP-SUBSET-057 — Multiple calls on a mutual edge

All interpreter modes shall prove a mutual-cycle member whose action contains
multiple calls only when every discovered call names the same next member and
independently progresses according to `TOPAL-FUNCTION-RECURSION-ALL-CALLS-001`.
Test traces and reversible debugger history shall retain every resulting
descent in evaluation order.

## TOPAL-INTP-SUBSET-058 — Rational natural exponentiation

All interpreter modes shall evaluate `Rational ^ Nat` exactly according to
`TOPAL-NUM-RAT-POW-001`, including the zero-exponent empty product. Negative
exponents shall remain inapplicable. Test traces and reversible debugger
history shall identify the Rational overload and exact evaluation rule.

## TOPAL-INTP-SUBSET-059 — Nat function classification

All interpreter modes shall accept `Nat` in implemented function parameter and
result classifiers according to `TOPAL-NUM-NAT-001`. Calls shall preserve exact
nonnegative `Int` values and reject negative arguments before function entry;
negative results shall fail the ordinary result validation. Test traces and
reversible debugger history shall retain the selected `Nat` signature.

## TOPAL-INTP-SUBSET-060 — Proven decreasing Nat recursion

All interpreter modes shall execute range-preserving decreasing `Nat` recursion only
after proving `TOPAL-FUNCTION-RECURSION-NAT-001`. The proof shall require a
nonnegative inclusive lower bound and a positive literal step no greater than
the bound plus one, preserving `Nat` at every recursive call.
Test traces and reversible debugger history shall expose proof acceptance and
each descent.

## TOPAL-INTP-SUBSET-061 — Proven increasing Nat recursion

All interpreter modes shall execute increasing `Nat` recursion only after
proving `TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001`. Every recursive edge
shall add a positive literal step toward an inclusive upper bound. Test traces
and reversible debugger history shall expose the proof and every descent.

## TOPAL-INTP-SUBSET-062 — Proven mutual decreasing Nat recursion

All interpreter modes shall execute a mutually decreasing `Nat` cycle only
after proving every member under `TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001`.
Every edge shall preserve `Nat` through a bounded decrement toward a nonnegative
bound. Test traces and reversible debugger history shall expose cycle proof and
descent.

## TOPAL-INTP-SUBSET-063 — Proven mutual increasing Nat recursion

All interpreter modes shall execute a mutually increasing `Nat` cycle only
after proving every member under
`TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001`. Test traces and reversible
debugger history shall expose completed cycle proof and every descent.

## TOPAL-INTP-SUBSET-064 — Bound-preserving Nat decrement steps

Direct and mutual decreasing `Nat` proofs shall accept a positive literal step
exactly when it is no greater than the nonnegative bound plus one. Larger steps
shall remain unproven because some admitted argument could cross below zero.

## TOPAL-INTP-SUBSET-065 — Payload-free enum values

All interpreter modes shall declare, resolve, display, and compare payload-free
nominal enum alternatives according to `TOPAL-TYPE-ENUM-001`. Duplicate labels
or collisions with existing names shall be diagnosed. Test traces and
reversible debugger history shall expose enum declaration and alternative use.

## TOPAL-INTP-SUBSET-066 — Enum function classifiers

All interpreter modes shall accept a visible declared enum name as an
implemented function parameter or result classifier. Calls and returns shall
validate nominal enum identity under `TOPAL-TYPE-ENUM-001`; test traces and
reversible debugger history shall retain the enum signature.

## TOPAL-INTP-SUBSET-067 — Exhaustive enum decisions

All interpreter modes shall execute named enum alternative matchers according
to `TOPAL-DECISION-ENUM-001`, prove complete coverage when `otherwise` is
absent, and delay unselected actions. Test traces and reversible debugger
history shall expose rule consideration and selection.

## TOPAL-INTP-SUBSET-068 — Arithmetic error-code namespace

All interpreter modes shall resolve the four `ArithmeticErrorCode` values from
the qualified `lang arithmetic` namespace according to
`TOPAL-NUM-ARITHMETIC-ERROR-001`. Resolution shall not invent or assign an
`Error.domain`. Test traces and reversible debugger history shall expose the
qualified namespace selection.

## TOPAL-INTP-SUBSET-069 — Successful Result contracts

All interpreter modes shall parse an explicit
`Result ( T, lang arithmetic ArithmeticErrorCode )` function result and accept
an ordinary successful `T` value under `TOPAL-TYPE-RESULT-001`. Declaration
shall neither construct an `Error` nor assign `Error.domain`. Test traces and
debugger history shall retain the complete result contract.

## TOPAL-INTP-SUBSET-070 — Dynamic Rational division failure

All interpreter modes shall return a structured arithmetic `Error` for dynamic
Rational division by zero under `TOPAL-NUM-DYNAMIC-DIVZERO-001`, while retaining
the static-zero diagnostic. Test traces and reversible debugger history shall
expose code construction, reporting-overload domain, and source provenance.

## TOPAL-INTP-SUBSET-071 — Negative Rational exponentiation

All interpreter modes shall evaluate a statically nonzero Rational base raised
to a negative Int exponent exactly under `TOPAL-NUM-RAT-NEG-POW-001`. A known
zero base shall remain a division-by-zero diagnostic. Test traces and reversible
debugger history shall distinguish the Rational/Int overload.

## TOPAL-INTP-SUBSET-072 — Dynamic negative-power zero failure

Within an arithmetic Result contract, all interpreter modes shall return a
structured division-by-zero Error when a dynamic Rational base is zero and its
exponent is negative. Domain and source provenance shall identify the reporting
power overload and base occurrence independently.

## TOPAL-INTP-SUBSET-073 — Structured Error propagation

All interpreter modes shall propagate an arithmetic Error through a compatible
fallible function result without reconstructing it. Test traces and reversible
debugger history shall distinguish initial construction from each propagation
boundary and retain code and domain.

## TOPAL-INTP-SUBSET-074 — Exhaustive Result decisions

All interpreter modes shall execute exhaustive `Ok value` and `Error problem`
decisions under `TOPAL-DECISION-RESULT-001`, bind only the selected payload,
and delay the other action. Test traces and reversible debugger history shall
expose matcher consideration, selection, and payload binding.

## TOPAL-INTP-SUBSET-075 — Structured Error field selection

All interpreter modes shall select `code` and `domain` from a structured Error
under `TOPAL-ERROR-FIELD-001`. The code shall retain its namespace-defined
concrete `ErrorCode` subtype while the compiler-derived reporting domain remains
a distinct `ErrorDomain`. Test traces and reversible debugger history shall
expose each selection.

## TOPAL-INTP-SUBSET-076 — Qualified Error-code decisions

All interpreter modes shall match qualified arithmetic code patterns within
Result decisions under `TOPAL-DECISION-ERROR-CODE-001`, without publishing code
identifiers into global scope or consulting `Error.domain`. Test traces and
reversible debugger history shall expose code-pattern selection.

## TOPAL-INTP-SUBSET-077 — Classified-binding Result projection

All interpreter modes shall project a Result when a binding explicitly
requires its successful classifier under `TOPAL-TYPE-RESULT-PROJECT-001`.
Success shall create the binding; failure shall return the complete Error from
the enclosing compatible fallible function without executing later statements.
Test traces and reversible debugger history shall distinguish both paths.
Failure projection from an infallible or top-level context shall produce a
source-located diagnostic with actionable help rather than a later generic
result-classifier mismatch.

## TOPAL-INTP-SUBSET-078 — Exhaustive arithmetic-code decisions

All interpreter modes shall accept one `Ok` matcher plus all four qualified
`lang arithmetic ArithmeticErrorCode` alternatives as an exhaustive Result
decision without a generic Error fallback. Missing alternatives remain an
incomplete-decision diagnostic. Test traces and reversible debugger history
shall expose the selected qualified code.
The incomplete diagnostic shall list the missing qualified alternatives and
offer both exhaustive patterns and a generic Error fallback as repairs.
Repeated qualified code patterns shall be diagnosed at the later occurrence
with actionable help and shall never count toward exhaustiveness.
Qualified code patterns after a generic Error matcher shall be diagnosed as
unreachable with help describing the required specific-before-fallback order.

## TOPAL-INTP-SUBSET-079 — Decision fallback reachability

The shared frontend used by every interpreter mode shall reject decision rules
after `otherwise` as unreachable, point to the first unreachable matcher, and
provide actionable ordering help. LSP diagnostics shall preserve the same code,
message, and source range.

## TOPAL-INTP-SUBSET-080 — Character classification

All interpreter modes shall accept `Character` in classified bindings,
function parameters, and results under `TOPAL-STRING-CHARACTER-CLASSIFIER-001`.
Classification shall use the pinned Unicode character segmentation and retain
the original String sequence. LSP validation and reversible debugger history
shall cover the same commented examples.
Failed classification shall report the observed user-perceived character count
at the initializer and provide actionable one-character guidance.

## TOPAL-INTP-SUBSET-081 — String construction from Character

All interpreter modes shall implement `String character` under
`TOPAL-STRING-FROM-CHARACTER-001`, preserving the Character's exact Unicode
sequence. Test traces and reversible debugger history shall expose construction
separately from literal evaluation and classification.

## TOPAL-INTP-SUBSET-082 — Euclidean Int modulo

All interpreter modes shall implement `%` for Int operands under
`TOPAL-NUM-INT-MODULO-001`, including negative operands. Literal zero shall be
a source diagnostic; dynamic zero within an arithmetic Result contract shall
construct a structured Error with the modulo overload domain. LSP highlighting
and reversible debugger history shall expose the operator and decision trace.

## TOPAL-INTP-SUBSET-083 — Euclidean Int quotient and modulo

All interpreter modes shall implement `/%` for Int operands under
`TOPAL-NUM-INT-QUOTIENT-MODULO-001` and classify its product success through
`Result ((Int, Int), lang arithmetic ArithmeticErrorCode)`. Literal and dynamic
zero handling, LSP highlighting, test traces, and reversible debugger history
shall parallel `%` while retaining the distinct reporting overload.

## TOPAL-INTP-SUBSET-084 — Exact numeric absolute value

All interpreter modes shall implement `absolute` for finite Int and Rational
under `TOPAL-NUM-ABS-001`, retaining the operand domain. LSP completion, test
traces, reversible debugger history, and a commented example shall expose both
overload selections.

## TOPAL-INTP-SUBSET-085 — Named exact numeric negation

All interpreter modes shall implement `negate` for finite Int and Rational
under `TOPAL-NUM-NEG-001` and `TOPAL-NUM-RAT-NEG-001`. Results shall equal
prefix negation while traces distinguish named root overload selection. LSP
completion and reversible debugger history shall cover both domains.

## TOPAL-INTP-SUBSET-086 — Exact numeric zero construction

All interpreter modes shall implement `zero Int` and `zero Rational` under
`TOPAL-NUM-ZERO-001`, retaining the explicitly named domain. LSP completion,
test traces, reversible debugger history, and a commented example shall expose
both type-directed root selections.

## TOPAL-INTP-SUBSET-087 — Exact numeric one construction

All interpreter modes shall implement `one Int` and `one Rational` under
`TOPAL-NUM-ONE-001`, retaining the explicitly named domain. LSP completion,
test traces, reversible debugger history, and the numeric-identity example
shall expose both type-directed root selections.

## TOPAL-INTP-SUBSET-088 — Exact three-way comparison

All interpreter modes shall implement `<=>` for finite Int and Rational under
`TOPAL-NUM-THREE-WAY-COMPARE-001`, including canonical mixed-domain conversion,
and return nominal `Comparison` values. LSP highlighting, test traces,
reversible debugger history, and a commented example shall expose all three
alternatives and overload selection.

## TOPAL-INTP-SUBSET-089 — Exhaustive Comparison decisions

All interpreter modes shall execute exhaustive `Less`, `Equal`, and `Greater`
decisions over the built-in nominal `Comparison` enum under
`TOPAL-DECISION-ENUM-001`. The alternatives shall not require a source Enum
declaration. Test traces and reversible debugger history shall expose matcher
consideration and selection; LSP validation shall accept the example.

## TOPAL-INTP-SUBSET-090 — Nat numeric identities

All interpreter modes shall extend `zero` and `one` to the supported Nat
refinement under `TOPAL-NUM-ZERO-001` and `TOPAL-NUM-ONE-001`. Results shall be
exact nonnegative Int values satisfying Nat, while traces retain Nat-specific
root selection. LSP validation and reversible debugger history shall cover the
updated commented identity example.

## TOPAL-INTP-SUBSET-091 — Closed exact Rational-to-Int narrowing

All interpreter modes shall allow a statically closed Rational result with
canonical denominator one to satisfy an explicitly Int-classified binding under
`TOPAL-NUM-RATIONAL-INT-EXACT-001`. Non-integral closed results shall produce a
source-located diagnostic with actionable help. Test traces and reversible
debugger history shall expose the exact conversion; LSP validation shall accept
the commented example.

## TOPAL-INTP-SUBSET-092 — Dynamic Rational-to-Int validation

All interpreter modes shall validate a dynamically obtained Rational at an
Int-classified binding in a compatible arithmetic Result function under
`TOPAL-NUM-RATIONAL-INT-VALIDATE-001`. Exact integers shall succeed without
rounding; other finite rationals shall propagate `not-representable` with the
compiler-derived `root.Int(Rational)` reporting domain. Test traces and
reversible debugger history shall expose both outcomes; LSP validation shall
accept the commented example.

## TOPAL-INTP-SUBSET-093 — Exact checked Int construction

All interpreter modes shall execute prefix `Int value` checked construction
under `TOPAL-NUM-INT-CONSTRUCT-001`. Int identity and exactly integral Rational
operands shall succeed; a closed fractional Rational shall be diagnosed and a
dynamic fractional Rational shall return `not-representable`. Test traces and
reversible debugger history shall expose construction decisions; LSP validation
shall accept the commented example.

## TOPAL-INTP-SUBSET-094 — Checked Nat constraint construction

All interpreter modes shall execute prefix `Nat value` validation under
`TOPAL-NUM-NAT-CONSTRUCT-001`. Nonnegative Int operands shall succeed, closed
negative operands shall produce an actionable diagnostic, and dynamic negative
operands shall return `out-of-range` from `root.Nat(Int)`. Test traces and
reversible debugger history shall expose success and failure; LSP validation
shall accept the commented example.

## TOPAL-INTP-SUBSET-095 — Closed finite Rational construction

All interpreter modes shall execute `Rational (numerator, denominator)` for
closed finite Int components under `TOPAL-NUM-RATIONAL-CONSTRUCT-001` and expose
canonical sign, greatest-common-divisor, and zero normalization. A closed zero
denominator shall produce an actionable diagnostic. Test traces and reversible
debugger history shall expose construction; LSP validation shall accept the
commented example.

## TOPAL-INTP-SUBSET-096 — Dynamic Rational construction

All interpreter modes shall execute dynamic finite Rational construction under
`TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001`. Nonzero denominators shall produce
canonical values; directionless zero shall return `division-by-zero` for a
nonzero numerator and `indeterminate` for zero. Both failures shall use
`root.Rational(Int,Int)`. Test traces and reversible debugger history shall
expose every outcome; LSP validation shall accept the commented example.

## TOPAL-INTP-SUBSET-097 — Explicit Rational construction from Int

All interpreter modes shall execute prefix `Rational value` for an Int operand
as the total canonical embedding under `TOPAL-NUM-INT-RATIONAL-CONVERT-001`.
The result shall have denominator one without a Result wrapper. Test traces and
reversible debugger history shall expose explicit construction; LSP validation
shall accept the updated commented Rational construction example.

## TOPAL-INTP-SUBSET-098 — Inclusive Int ranges

All interpreter modes shall construct `lower .. upper` as a closed Int range
under `TOPAL-RANGE-INCLUSIVE-001`, including an empty value when bounds are
reversed. Test traces and reversible debugger history shall distinguish
nonempty and empty construction; LSP validation and highlighting shall accept
the commented example.

## TOPAL-INTP-SUBSET-099 — Int range membership

All interpreter modes shall execute both `value in interval` and
`interval contains value` under `TOPAL-RANGE-MEMBERSHIP-001`, with equivalent
Boolean results and rejection from empty ranges. Test traces and reversible
debugger history shall expose accepted and rejected decisions; LSP validation
shall accept the updated commented range example.

## TOPAL-INTP-SUBSET-100 — Rational ranges

All interpreter modes shall construct and test closed Rational ranges under
`TOPAL-RANGE-RATIONAL-001`, including mixed Int endpoints and Int membership via
the canonical exact conversion. Test traces and reversible debugger history
shall expose range and conversion decisions; LSP validation shall accept the
commented example.

## TOPAL-INTP-SUBSET-101 — Range function classifiers

All interpreter modes shall accept `Range Int` and `Range Rational` classifiers
at ordinary function boundaries under `TOPAL-RANGE-CLASSIFIER-001`. Calls shall
preserve the endpoint domain and support membership inside the callee. Test
traces and reversible debugger history shall expose the calls; LSP validation
shall accept the updated commented range examples.

## TOPAL-INTP-SUBSET-102 — Range function results

All interpreter modes shall accept `Range Int` and `Range Rational` as ordinary
function result classifiers under `TOPAL-RANGE-CLASSIFIER-001`. Returned ranges
shall retain their endpoint domain and remain usable for membership. Parser,
LSP, trace, and reversible debugger coverage shall exercise both domains in the
updated commented examples.

## TOPAL-INTP-SUBSET-103 — Range intersection

All interpreter modes shall execute `left and right` for matching Int or
Rational range domains under `TOPAL-RANGE-INTERSECTION-001`. Overlapping inputs
shall narrow both bounds and disjoint inputs shall produce an empty range. Test
traces and reversible debugger history shall expose intersection construction;
LSP validation shall accept the updated commented examples.

## TOPAL-INTP-SUBSET-104 — Boolean negation

All interpreter modes shall execute `not value` for Boolean operands under
`TOPAL-TYPE-BOOLEAN-LOGIC-001` without numeric coercion. Traces and reversible
debugger history shall expose the selected root operation and logical decision;
LSP validation shall accept the commented example.

## TOPAL-INTP-SUBSET-105 — Eager Boolean conjunction

All interpreter modes shall execute `left and right` for Boolean operands under
`TOPAL-TYPE-BOOLEAN-LOGIC-001`, evaluating both operands exactly once from left
to right without short-circuiting. Range intersection shall remain a distinct
overload. Traces, LSP validation, and reversible debugger history shall cover
the updated commented truth-table example.

## TOPAL-INTP-SUBSET-106 — Eager Boolean disjunction

All interpreter modes shall execute `left or right` for Boolean operands under
`TOPAL-TYPE-BOOLEAN-LOGIC-001`, evaluating both operands exactly once from left
to right without short-circuiting. Range operands shall not be mistaken for a
Range-returning overload. Traces, LSP validation, and reversible debugger
history shall cover the updated commented truth-table example.

## TOPAL-INTP-SUBSET-107 — Eager Boolean exclusive disjunction

All interpreter modes shall execute `left xor right` for Boolean operands under
`TOPAL-TYPE-BOOLEAN-LOGIC-001`, evaluating both operands exactly once from left
to right and returning true exactly when they differ. Traces, LSP validation,
and reversible debugger history shall cover the completed commented logical
truth-table example.

## TOPAL-INTP-SUBSET-108 — Explicit Optional construction

All interpreter modes shall execute `Some value` and `None T` under
`TOPAL-TYPE-OPTIONAL-CONSTRUCT-001`, preserving the nominal payload classifier
and displaying the approved constructor spellings. Some payloads shall evaluate
exactly once. Traces, LSP validation, and reversible debugger history shall
cover the commented example.

## TOPAL-INTP-SUBSET-109 — Contextual None bindings

All interpreter modes shall allow bare `None` in an immediately
`Optional T`-classified binding under `TOPAL-TYPE-OPTIONAL-CONTEXT-001`, retain
the nominal payload classifier, and continue rejecting uncontextual bare None.
Traces, LSP validation, and reversible debugger history shall cover the updated
commented example.

## TOPAL-INTP-SUBSET-110 — Optional function boundaries

All interpreter modes shall accept `Optional T` parameters and results under
`TOPAL-TYPE-OPTIONAL-BOUNDARY-001`, preserving nominal payload identity for both
Some and None alternatives. Traces and reversible debugger history shall expose
ordinary calls; LSP validation shall accept the updated commented example.

## TOPAL-INTP-SUBSET-111 — Contextual None function results

All interpreter modes shall infer bare `None` from an `Optional T` function
result for both final expressions and explicit returns under
`TOPAL-TYPE-OPTIONAL-CONTEXT-001`. Traces and reversible debugger history shall
expose the same contextual construction rule; LSP validation shall accept the
updated commented example.

## TOPAL-INTP-SUBSET-112 — Exhaustive Optional decisions

All interpreter modes shall execute complete `Some payload` and `None` decision
tables under `TOPAL-DECISION-OPTIONAL-001`, bind a present payload only in its
selected branch, and reject incomplete tables. Traces and reversible debugger
history shall expose rule consideration, selection, and payload binding; LSP
validation shall accept the updated commented example.

## TOPAL-INTP-SUBSET-113 — Derived Optional equality

All interpreter modes shall execute equality and inequality for matching
`Optional T` values when T provides equality under
`TOPAL-TYPE-OPTIONAL-EQUALITY-001`. Some payloads shall compare recursively,
None shall compare by its nominal Optional identity, and mismatched classifiers
shall be rejected. Traces and reversible debugger history shall expose Optional
equality; LSP validation shall accept the updated commented example.

## TOPAL-INTP-SUBSET-114 — Optional String character indexing

All interpreter modes shall execute `text character-at index` under
`TOPAL-STRING-CHARACTER-AT-001`, returning complete pinned-Unicode grapheme
clusters as `Some Character` and negative or out-of-range indexes as None.
Traces and reversible debugger history shall expose present and absent results;
LSP validation shall accept the commented example.

## TOPAL-INTP-SUBSET-115 — Consumption of indexed characters

All interpreter modes shall pass the `Optional Character` produced by String
indexing through ordinary function boundaries and exhaustive Optional
decisions. A present payload shall retain its `Character` classifier for exact
`String` construction; an absent result shall select the `None` action. Traces
and reversible debugger history shall expose the indexing, decision, and
construction rules, and LSP validation shall accept the commented example.

## TOPAL-INTP-SUBSET-116 — Universal String uppercase

All interpreter modes shall execute `upper text` for plain String values under
`TOPAL-STRING-UPPER-001`, using deterministic Unicode default casing without
ambient locale. Traces and reversible debugger history shall expose overload
selection and transformation; LSP completion and validation shall cover the
commented example.

## TOPAL-INTP-SUBSET-117 — Universal String lowercase

All interpreter modes shall execute `lower text` for plain String values under
`TOPAL-STRING-LOWER-001`, using deterministic Unicode default casing without
ambient locale. Traces and reversible debugger history shall expose overload
selection and transformation; LSP completion and validation shall cover the
commented example.

## TOPAL-INTP-SUBSET-118 — Universal String case folding

All interpreter modes shall execute `case-fold text` for plain String values
under `TOPAL-STRING-CASE-FOLD-001`, using full deterministic Unicode default
folding without ambient locale. Traces and reversible debugger history shall
expose overload selection and transformation; LSP completion and validation
shall cover the commented example.

## TOPAL-INTP-SUBSET-119 — Canonical String equality

All interpreter modes shall execute `left canonically-equals right` for plain
String values under `TOPAL-STRING-CANONICAL-EQUALITY-001`, without changing
exact `=` semantics or either operand. Traces and reversible debugger history
shall expose overload selection and the comparison reason; LSP completion and
validation shall cover the commented example.

## TOPAL-INTP-SUBSET-120 — String Character traversal collection

All interpreter modes shall execute `characters text collect String` under
`TOPAL-STRING-CHARACTERS-COLLECT-001`, yielding complete grapheme clusters in
order and reconstructing the exact preserved text. Formal traces and reversible
debugger history shall expose each yield and collection; LSP completion and
validation shall cover the commented example.

## TOPAL-INTP-SUBSET-121 — String Character generator signature

All tools shall classify `characters text` as
`Generator Character Unit Unit`: each yield is a complete Character, resumption
accepts Unit, and normal exhaustion returns Unit. LSP completion and validation,
formal traces, and reversible debugger history shall use the same signature.

## TOPAL-INTP-SUBSET-122 — Direct String Character foreach

All interpreter modes shall consume `characters text` with direct `foreach`
under `TOPAL-STRING-CHARACTERS-FOREACH-001`, binding each Character in order,
resuming with Unit, and returning Unit on exhaustion. Formal traces and the
reversible debugger shall expose yields, resumptions, and final return; LSP
validation shall accept the commented example.

## TOPAL-INTP-SUBSET-123 — Named linear String Character generator

All interpreter modes shall bind `characters text` as
`Generator Character Unit Unit` and consume the binding linearly with `foreach`
under `TOPAL-STRING-CHARACTERS-GENERATOR-001`. Traces and reversible debugger
history shall distinguish generator start from source-level consumption; LSP
validation shall accept the commented example.

## TOPAL-INTP-SUBSET-124 — Explicit String Character generator classification

All tools shall accept `Generator Character Unit Unit` as the explicit
classifier of `characters text` under `TOPAL-STRING-CHARACTERS-CLASSIFIER-001`.
Classification shall preserve linear consumption, traces, LSP validation, and
reversible debugger behavior in the updated commented example.

## TOPAL-INTP-SUBSET-125 — String Character generator function results

All interpreter modes shall return fresh `Generator Character Unit Unit` values
from ordinary functions under `TOPAL-STRING-CHARACTERS-RESULT-001`. The caller
shall receive one linearly consumable continuation. Traces, LSP validation, and
reversible debugger history shall cover the commented example.

## TOPAL-INTP-SUBSET-126 — Consumed generator diagnostics

All interpreter modes shall reject reuse of a consumed named generator under
`TOPAL-STRING-CHARACTERS-LINEAR-001` with a source-located diagnostic that names
the binding and suggests constructing a fresh generator. Reversible debugger
history shall retain the earlier consumption and the later failure; the
commented failing debugger example shall remain executable as a scripted test.

## TOPAL-INTP-SUBSET-127 — Unit-returning foreach actions

All interpreter modes shall require a direct Character `foreach` action to
return Unit under `TOPAL-STRING-CHARACTERS-FOREACH-001`. Non-Unit results shall
produce a source-located diagnostic rather than being silently discarded. The
commented examples shall discard demonstration conversions explicitly; LSP and
debugger validation shall accept the corrected examples.

## TOPAL-INTP-SUBSET-128 — String Character generator parameters

All interpreter modes shall transfer a named `Generator Character Unit Unit`
binding into a matching ordinary function parameter under
`TOPAL-STRING-CHARACTERS-PARAMETER-001`. The caller binding shall be consumed,
and the callee may traverse the single continuation. Formal traces, LSP
validation, and reversible debugger history shall cover the commented example.

## TOPAL-INTP-SUBSET-129 — Abandoned String Character generator closure

All interpreter modes shall close a transferred but unconsumed
`Generator Character Unit Unit` parameter at function-scope exit under
`TOPAL-STRING-CHARACTERS-CLOSE-001`. The caller binding shall remain consumed;
formal traces and reversible debugger history shall distinguish close from
ordinary exhaustion, and LSP validation shall accept the commented example.

## TOPAL-INTP-SUBSET-130 — Generator error-code vocabulary

All tools shall recognize `lang generator GeneratorErrorCode` and its initial
`generator-closed` alternative under `TOPAL-GENERATOR-ERROR-CODE-001`, keeping
that namespace distinct from compiler-derived `Error.domain`. Interpreter
traces, LSP validation, and reversible debugger history shall cover the
commented example.

## TOPAL-INTP-SUBSET-131 — Generator close domain trace

When the built-in Character generator handles abandonment in the root lexical
namespace, formal traces and reversible debugger history shall report domain
`root` and code `generator-closed`, while retaining `root.characters` as
separate generator provenance. Handling shall still finish with Unit.

## TOPAL-INTP-SUBSET-132 — Named single-yield generators

All interpreter modes shall declare and apply the first custom generator subset
under `TOPAL-GENERATOR-DECLARATION-001`: one Character input, one discarded
Character yield, Unit resumption, and Unit return. Direct foreach shall consume
the resulting linear generator. Traces, LSP highlighting and diagnostics, and
reversible scripted debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-133 — Multiple custom generator yields

All interpreter modes shall execute multiple discarded Character yields in
source order under `TOPAL-GENERATOR-FOREACH-001`. Test traces and reversible
scripted-debugger history shall expose each yield and Unit resumption, while LSP
validation shall accept the updated commented examples.

## TOPAL-INTP-SUBSET-134 — Custom generator local bindings

All interpreter modes shall evaluate ordinary local bindings between custom
generator yields under `TOPAL-GENERATOR-LOCAL-BINDING-001`. Later yields shall
observe those bindings without leaking them to the caller. LSP validation,
formal traces, and reversible scripted-debugger history shall cover a commented
example.

## TOPAL-INTP-SUBSET-150 — Distinct final String from generator

All interpreter modes shall traverse `Generator String Unit String`, preserving
each yielded String and producing its distinct final String under
`TOPAL-GENERATOR-FINAL-RETURN-001`. Formal traces, LSP validation, and reversible
scripted-debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-145 — Unconsumed custom generator parameter closure

When a function does not consume its transferred custom Generator parameter,
all interpreter modes shall close that suspended continuation at function exit
under `TOPAL-GENERATOR-CLOSE-001`. The caller binding shall remain consumed;
formal traces shall retain lexical root domain separately from custom generator
provenance. LSP validation and reversible scripted-debugger history shall cover
a commented example.

## TOPAL-INTP-SUBSET-152 — Explicit custom generator return

All interpreter modes shall execute `return value` inside a custom generator
under `TOPAL-GENERATOR-EXPLICIT-RETURN-001`, including return before the first
yield. The returned value shall satisfy the declared final classifier and become
the foreach result. LSP validation and reversible scripted-debugger history
shall cover a commented example.

## TOPAL-INTP-SUBSET-153 — Explicit return after generator resumption

All interpreter modes shall preserve custom generator continuation state when
an explicit return follows a yield. Traversal shall invoke the action, resume
the generator, and then expose the declared final value under
`TOPAL-GENERATOR-EXPLICIT-RETURN-001`. LSP validation and reversible scripted-
debugger history shall cover a commented example and its trace ordering.

## TOPAL-INTP-SUBSET-154 — Boolean custom generator values

All interpreter modes shall support Boolean custom-generator input, yield, and
final-return directions under `TOPAL-GENERATOR-DECLARATION-001`. Direct foreach
shall bind each Boolean yield to its action and produce the distinct final
Boolean. Formal traces, LSP validation, and reversible scripted-debugger
history shall cover a commented example.

## TOPAL-INTP-SUBSET-155 — Int custom generator values

All interpreter modes shall support arbitrary-precision Int custom-generator
input, yield, and final-return directions. Suspension and resumption shall
preserve the numeric value without narrowing. Formal traces, LSP validation,
and reversible scripted-debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-156 — Rational custom generator values

All interpreter modes shall support exact Rational custom-generator input,
yield, and final-return directions. Suspension shall preserve the canonical
numeric value without conversion through a finite representation. Formal
traces, LSP validation, and reversible scripted-debugger history shall cover a
commented example.

## TOPAL-INTP-SUBSET-157 — Unit custom generator values

All interpreter modes shall support Unit as a custom-generator input and yield
classifier. Foreach shall invoke its action for each yielded Unit and resume the
continuation with Unit. LSP validation and reversible scripted-debugger history
shall cover a commented example.

## TOPAL-INTP-SUBSET-158 — Optional custom generator values

All interpreter modes shall support Optional values as custom-generator input,
yield, and final return when their payload classifier is supported. Suspension
shall preserve both the alternative and nominal payload classifier. Formal
traces, LSP validation, and reversible scripted-debugger history shall cover a
commented example.

## TOPAL-INTP-SUBSET-159 — Range custom generator values

All interpreter modes shall support exact Range Int and Range Rational values
as custom-generator input, yield, and final return. Suspension shall preserve
inclusive endpoints and canonical empty state. Formal traces, LSP validation,
and reversible scripted-debugger history shall cover a commented Range Int
example.

## TOPAL-INTP-SUBSET-160 — Nat custom generator values

All interpreter modes shall support Nat as a custom-generator input, yield, and
final-return classifier. Every transferred value shall satisfy the nonnegative
constraint. Formal traces, LSP validation, and reversible scripted-debugger
history shall cover a commented example.

## TOPAL-INTP-SUBSET-161 — Enum custom generator values

All interpreter modes shall support a declared nominal enum as a custom-
generator input, yield, and final-return classifier. Suspension shall preserve
both enum identity and alternative. Formal traces, LSP validation, and
reversible scripted-debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-162 — Positional product custom generator values

All interpreter modes shall support positional products of supported component
classifiers as custom-generator input, yield, and final return. Suspension shall
preserve component order and classifiers. Syntax, formal traces, LSP validation,
and reversible scripted-debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-163 — Result custom generator values

All interpreter modes shall support arithmetic Result values as custom-
generator input, yield, and final return when their success classifier is
supported. Structured Error domain, code, and source position shall survive the
boundary. Syntax, LSP, formal trace, and reversible debugger coverage shall use
a commented example.

## TOPAL-INTP-SUBSET-164 — Comparison custom generator values

All interpreter modes shall support the language-defined nominal Comparison as
a custom-generator input, yield, and final-return classifier. Suspension shall
preserve nominal identity and alternative. Formal traces, LSP validation, and
reversible scripted-debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-165 — Nested Optional-product generator values

All interpreter modes shall recursively validate `Optional (A, B)` custom-
generator input, yield, and final-return classifiers when both components are
supported. Parser, LSP, formal trace, and reversible debugger coverage shall
verify preservation of Optional and product structure using a commented
example.

## TOPAL-INTP-SUBSET-166 — Nested Result-product generator values

All interpreter modes shall recursively validate an arithmetic Result whose
success classifier is a supported positional product at custom-generator input,
yield, and final-return boundaries. LSP, formal trace, and reversible debugger
coverage shall verify product structure and exact component values using a
commented example.

## TOPAL-INTP-SUBSET-167 — Absent nested Optional values

All interpreter modes shall construct `None (A, B)` with its complete
positional-product payload classifier and transfer it through supported custom-
generator boundaries. LSP, formal trace, and reversible debugger coverage shall
use a commented example.

## TOPAL-INTP-SUBSET-168 — Recursive nominal generator classifiers

All interpreter modes shall recursively recognize declared nominal enums inside
Optional and arithmetic Result classifiers at custom-generator input, yield,
and final-return boundaries. The shared classifier validation used by ordinary
functions and generators shall preserve the nominal enum identity. LSP, formal
trace, and reversible debugger coverage shall use a commented example.

## TOPAL-INTP-SUBSET-169 — Generator final decisions and diagnostics

All interpreter modes shall evaluate a final decision table after generator
resumption and use its selected action as the final return. Formal trace and
reversible debugger history shall expose resumption, decision selection, and
return in order. Generator yield and return mismatches shall report both the
expected and structurally found classifiers with actionable help. LSP and a
commented example shall cover the valid form.

## TOPAL-INTP-SUBSET-170 — Continuation-local declaration state

All interpreter modes shall retain generator-local enum and function
declarations, nominal identity, alternatives, and lexical captures across
suspension. Calls after resumption shall use the retained declarations, which
shall not escape to the consumer scope. The shared continuation snapshot used
by the scripted debugger shall retain this state reversibly. LSP, formal trace,
and commented examples shall cover the feature.

## TOPAL-INTP-SUBSET-171 — Local declarations during generator close

All interpreter modes shall restore continuation-local enum and function state
when abandonment delivers `generator-closed`. A matching close-result action
may call the retained function with a retained nominal alternative before the
generator finishes. Formal traces and reversible debugger history shall expose
close binding, decision selection, local call, and completion in order. LSP and
commented examples shall cover the feature.

## TOPAL-INTP-SUBSET-172 — Multi-input generator overloads

All interpreter modes shall declare, select, and apply generator overloads with
one or more initial operands under `TOPAL-GENERATOR-OVERLOAD-001`. Positional
product arguments shall bind in declaration order, selection shall be based on
the complete input classifier sequence, and duplicate signatures shall be
rejected. Diagnostics shall report the found argument classifier and available
signatures. Syntax, LSP, formal traces, reversible debugger history, and a
commented example shall cover unary and binary overloads.

## TOPAL-INTP-SUBSET-173 — Foreach final-result binding

All interpreter modes shall support binding the distinct final return of direct
generator foreach under `TOPAL-GENERATOR-FOREACH-RESULT-001`. The binding shall
be created after traversal completes and remain available to later statements.
An explicit result classifier shall be validated before binding and mismatch
diagnostics shall report expected and structurally found classifiers.
Syntax, LSP, formal traces, reversible debugger history, and the overload
example shall cover two differently classified final results.

## TOPAL-INTP-SUBSET-174 — Generic generator function boundaries

All interpreter modes shall transfer supported scalar custom-generator values
through ordinary function results and parameters under
`TOPAL-GENERATOR-FUNCTION-CLASSIFIER-001`. A `Generator Int Unit String` shall
retain its suspension state and final String through both boundaries. LSP,
formal traces, reversible debugger history, and a commented example shall cover
construction, return, parameter transfer, foreach result binding, and final
value propagation.

## TOPAL-INTP-SUBSET-175 — Compound generator function classifiers

All interpreter modes shall parse and transfer function parameters and results
classified as `Generator (A, B) Unit (C, D)`. Classifier parsing shall respect
balanced parentheses, and runtime validation shall preserve positional product
structure through suspension, function ownership transfer, traversal, and
final-result binding. Syntax, LSP, formal traces, reversible debugger history,
and a commented example shall cover the feature.

## TOPAL-INTP-SUBSET-176 — Nested generator function classifiers

All interpreter modes shall parse, validate, and transfer generator values whose
yield and return directions recursively compose supported `Optional`, product,
and `Result` classifiers. The parser shall preserve constructor boundaries when
whitespace occurs inside a direction classifier. Syntax, LSP, formal traces,
reversible debugger history, and a commented example shall cover the feature.

## TOPAL-INTP-SUBSET-177 — List construction and decomposition

All interpreter modes shall construct homogeneous `List T` values from `Empty`
and `Entry (value, remaining-list)` in an expected List context, compare Lists
structurally, and decompose them with complete Empty/Entry decisions. Entry
classifier and remainder errors shall carry precise source diagnostics and
actionable help. Syntax, LSP, formal traces, reversible debugger history, and a
commented example shall cover the feature.

## TOPAL-INTP-SUBSET-178 — Fundamental List operations

All interpreter modes shall support `prepend`, `append`, and `concat` without
changing List order or element classifiers, and shall report `entry-count` and
`empty?` for Lists. Incompatible entry and List classifiers shall produce
source-located diagnostics. LSP, formal traces, reversible debugger history,
and the commented List example shall cover every operation.

## TOPAL-INTP-SUBSET-179 — Explicit empty and singleton Lists

All interpreter modes shall construct `empty List T` without contextual type
information and infer `List T` for `one value` from the value's structural
classifier. Numeric `one Type` overloads shall remain unchanged. LSP, formal
traces, reversible debugger history, and the commented List example shall cover
both constructors.

## TOPAL-INTP-SUBSET-180 — List generator values

All interpreter modes shall preserve `List T` values across custom generator
input, yield, suspension, final return, and ordinary function continuation
transfers when `T` is supported. Formal traces shall retain the exact
`Generator List T Unit List T` classifier. LSP, reversible debugger history,
and a commented example shall cover the feature.

## TOPAL-INTP-SUBSET-181 — Total List uncons

All interpreter modes shall return `None (T, List T)` when applying `uncons` to
an empty `List T`, and `Some (first, rest)` for a nonempty List while preserving
entry order and the `List T` classifier. LSP, formal traces, reversible debugger
history, diagnostics, and the commented List example shall cover the operation.

## TOPAL-INTP-SUBSET-182 — Total List first and rest

All interpreter modes shall return `Optional T` from `first List T` and
`Optional (List T)` from `rest List T`, producing None on Empty and preserving
the first value or ordered tail on Entry. LSP, formal traces, reversible
debugger history, diagnostics, and the commented List example shall cover both
projections.

## TOPAL-INTP-SUBSET-183 — Recursive List classifiers

All interpreter modes shall preserve Lists whose element classifier is a
supported product or another List through construction, function boundaries,
equality, and projection. LSP, formal traces, reversible debugger history, and
a commented example shall cover nested Lists of positional products.

## TOPAL-INTP-SUBSET-184 — List containment laws

All interpreter modes shall distinguish `contains-entry`, consecutive
`contains-sequence`, and gap-permitting ordered `contains-subsequence` for Lists.
Classifier and equality failures shall be source-located. LSP, formal traces,
reversible debugger history, and a commented example shall cover true and false
outcomes for all three laws.

## TOPAL-INTP-SUBSET-185 — List reversal

All interpreter modes shall reverse List order without changing its classifier,
entry count, or multiplicity. Reversing twice shall reproduce an equal List.
LSP, formal traces, reversible debugger history, and the commented List example
shall cover the operation.

## TOPAL-INTP-SUBSET-186 — Value-based List removal

All interpreter modes shall distinguish removing the first equal entry from
removing all equal entries while preserving retained order and the List
classifier. Classifier and equality failures shall be source-located. LSP,
formal traces, reversible debugger history, and a commented example shall cover
present, repeated, and absent target values.

## TOPAL-INTP-SUBSET-146 — Character-returning generator parameter

All interpreter modes shall transfer `Generator Character Unit Character` into
a matching ordinary function parameter, traverse its yields, and propagate the
distinct final Character as the function result. Formal traces shall combine
`TOPAL-GENERATOR-FUNCTION-PARAMETER-001` and
`TOPAL-GENERATOR-FINAL-RETURN-001`; LSP validation and reversible
scripted-debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-147 — Character-returning generator function result

All interpreter modes shall return a suspended `Generator Character Unit
Character` from an ordinary function without closing it, then let the caller
traverse its yields and receive its final Character. Formal traces shall combine
`TOPAL-GENERATOR-FUNCTION-RESULT-001` and
`TOPAL-GENERATOR-FINAL-RETURN-001`; LSP validation and reversible
scripted-debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-135 — Generator return before first yield

All interpreter modes shall support a custom generator that reaches its final
Unit before yielding under `TOPAL-GENERATOR-EARLY-RETURN-001`. Direct foreach
shall run no action and return Unit. Formal traces, LSP validation, and
reversible scripted-debugger history shall identify the final transition with
`TOPAL-GENERATOR-EARLY-RETURN-001` and cover a commented example.

## TOPAL-INTP-SUBSET-136 — Distinct generator final Character

All interpreter modes shall execute a custom `Generator Character Unit
Character` under `TOPAL-GENERATOR-FINAL-RETURN-001`. Direct foreach shall
consume every yield and produce the separately evaluated final Character.
Formal traces, LSP validation, and reversible scripted-debugger history shall
cover a commented example.

## TOPAL-INTP-SUBSET-148 — String initial input for custom generator

All interpreter modes shall accept a String initial parameter for a custom
`Generator Character Unit Unit`, evaluate String operations in its local scope,
and suspend at its Character yield. Formal traces, LSP validation, and
reversible scripted-debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-151 — Discarded generator-body computations

All interpreter modes shall execute an ordinary discarded computation between
custom generator yields under `TOPAL-GENERATOR-BODY-STATEMENT-001`. Formal
traces shall place it after the preceding Unit resumption and before the next
suspension. LSP validation and reversible scripted-debugger history shall cover
a commented example.

## TOPAL-INTP-SUBSET-149 — Custom String yields

All interpreter modes shall execute and traverse a custom `Generator String
Unit Unit`. Each yielded String shall reach the foreach action unchanged, and
each successful action shall resume the continuation with Unit. Formal traces,
LSP validation, and reversible scripted-debugger history shall cover a commented
example.

## TOPAL-INTP-SUBSET-139 — Abandoned custom generator trace

When function exit abandons a suspended custom generator, all interpreter modes
shall close it under `TOPAL-GENERATOR-CLOSE-001`. Formal traces shall report
lexical domain `root` and code `generator-closed`, retaining the qualified
custom generator name only as separate provenance. LSP validation and reversible
scripted-debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-144 — Custom generator function parameter

All interpreter modes shall transfer a suspended custom `Generator Character
Unit Unit` into an ordinary function parameter under
`TOPAL-GENERATOR-FUNCTION-PARAMETER-001`. The caller binding shall be consumed,
and the callee shall traverse the same continuation once. Formal traces, LSP
validation, and reversible scripted-debugger history shall cover a commented
example.

## TOPAL-INTP-SUBSET-140 — Custom generator close handler

All interpreter modes shall deliver `generator-closed` to a suspended custom
yield-result binding under `TOPAL-GENERATOR-CLOSE-HANDLER-001` and execute its
Error decision rule to final Unit. The Error domain shall be `root`; generator
identity shall remain separate provenance. Formal traces, LSP validation, and
reversible scripted-debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-141 — Yield-after-close diagnostic

All interpreter modes shall reject a custom generator that attempts another
yield after its suspended result receives `generator-closed`. The diagnostic
shall be source-located as `E-GENERATOR-YIELD-AFTER-CLOSE`. LSP syntax
validation and reversible scripted-debugger history shall cover the commented
failing example.

## TOPAL-INTP-SUBSET-142 — Qualified generator close-code pattern

All interpreter modes shall select `Error ( code is lang generator
generator-closed )` under `TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001` when a custom
yield receives the intrinsic close Error. Formal traces, LSP validation, and
reversible scripted-debugger history shall cover a commented example with
specific, generic Error, and Ok rules.

## TOPAL-INTP-SUBSET-143 — Custom generator function result

All interpreter modes shall return a suspended custom `Generator Character Unit
Unit` from an ordinary function under `TOPAL-GENERATOR-FUNCTION-RESULT-001`
without closing it at function exit. The caller shall receive and consume the
single continuation. Formal traces, LSP validation, and reversible
scripted-debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-137 — Custom generator suspension

All interpreter modes shall suspend custom execution at each yield under
`TOPAL-GENERATOR-SUSPEND-001`. A binding following a yield shall be evaluated
only after foreach records its Unit resumption and before the next suspension.
Formal traces, LSP validation, and reversible scripted-debugger history shall
verify that ordering with a commented example.

## TOPAL-INTP-SUBSET-138 — Unit resume-result binding

All interpreter modes shall bind a successful Unit resumption from `name is
yield value` under `TOPAL-GENERATOR-RESUME-BINDING-001`. The binding shall not
exist while suspended and shall be available to subsequent generator execution.
Formal traces, LSP validation, and reversible scripted-debugger history shall
cover a commented example.
## TOPAL-INTP-SUBSET-187 — Contextual anonymous List functions

All interpreter modes shall execute inferred anonymous functions used directly
by List `map`, `select`, and `fold`, including immutable lexical capture, under
`TOPAL-FUNCTION-ANONYMOUS-001`, `TOPAL-COLLECTION-MAP-001`,
`TOPAL-COLLECTION-SELECT-001`, and `TOPAL-COLLECTION-FOLD-001`. Formal traces,
LSP validation, and reversible scripted-debugger history shall cover commented
examples.
## TOPAL-INTP-SUBSET-188 — Complete ordered List sequence operations

All interpreter modes shall implement checked indexed insertion, regions and
removal; explicit zip policies and unzip; List traversal, predicate removal,
entry views, and List/String collection according to `TOPAL-LIST-*` and
`TOPAL-COLLECTION-*` rules in `spec/containers.md`. Invalid closed positions
shall be source diagnostics, while invalid unchecked runtime positions shall be
formal `out-of-range` Results whose domain identifies the lexical operation.
LSP validation and reversible scripted-debugger history shall cover commented
examples.
## TOPAL-INTP-SUBSET-189 — Fundamental Array, Set, Bag, and Map values

All interpreter modes shall collect finite Lists into fixed-count Arrays,
unique Sets, multiplicity-preserving Bags, and explicitly collision-resolved
Maps under `TOPAL-ARRAY-COLLECT-001`, `TOPAL-SET-COLLECT-001`,
`TOPAL-BAG-COLLECT-001`, and `TOPAL-MAP-COLLECT-001`. Counting, emptiness,
formal traces, LSP validation, and reversible scripted-debugger history shall
cover commented examples.
## TOPAL-INTP-SUBSET-190 — Recursive products and general sums

All interpreter modes shall construct and recursively classify nested tuples
and records, positional Variants, labeled Unions, payload-bearing alternatives,
and payload-free enum-like alternatives under `TOPAL-TYPE-PRODUCT-001`,
`TOPAL-TYPE-UNION-001`, `TOPAL-TYPE-VARIANT-001`, and
`TOPAL-DECISION-UNION-001`. Formal traces, LSP validation, and reversible
scripted-debugger history shall cover commented examples.
## TOPAL-INTP-SUBSET-191 — Constraints, conversions, equality, and ordering

All interpreter modes shall construct named constraints, validate closed and
dynamic values, retain successful evidence, forget evidence losslessly to the
base type, and derive base equality and ordering under
`TOPAL-TYPE-CONSTRAINT-001` and `TOPAL-TYPE-CONSTRAINT-VALIDATE-001`. Existing
nominal Enum, Union, and Variant identities shall remain distinct. Formal
traces, Rust-style rejection diagnostics, LSP validation, and reversible
scripted-debugger history shall cover commented examples.
## TOPAL-INTP-SUBSET-192 — Complete Optional and Result composition

All interpreter modes shall execute explicit/contextual Optional construction,
complete Optional and Result decisions, compatible contextual propagation, and
all structured Error field projections under `TOPAL-TYPE-OPTIONAL-*`,
`TOPAL-TYPE-RESULT-*`, and `TOPAL-ERROR-FIELD-001`. Formal traces, source-located
diagnostics, LSP validation, and reversible scripted-debugger history shall
cover commented examples.
## TOPAL-INTP-SUBSET-193 — Settled modular numeric families

All interpreter modes shall implement nominal ModNat and ModInt ranges, checked
construction, explicit modular reduction, wrapping arithmetic, equality, and
canonical ordering under `TOPAL-NUM-MODULAR-*`. Formal traces, Rust-style
closed-range diagnostics, LSP validation, and reversible scripted-debugger
history shall cover commented examples. Numeric families lacking approved
source construction syntax remain documented in the PR rather than receiving
an implementation-invented spelling.
## TOPAL-INTP-SUBSET-194 — Range selection and slicing evidence

All interpreter modes shall use Range Int as a convex value or zero-based index
predicate over Lists and as a Character-index predicate over String under
`TOPAL-RANGE-VALUE-SELECTION-001` and `TOPAL-RANGE-INDEX-SELECTION-001`.
Observable values shall preserve source kind, order, and multiplicity; formal
traces shall retain selection provenance without exposing representation. LSP
validation and reversible scripted-debugger history shall cover commented
examples.

## TOPAL-INTP-SUBSET-195 — Explicit completion evidence

All interpreter modes shall evaluate `Completed` as zero-data completion
evidence distinct from `Unit` under `TOPAL-EXEC-COMPLETED-001`. Formal traces,
LSP validation, and reversible scripted-debugger history shall cover a
commented function example.

## TOPAL-INTP-SUBSET-196 — Immutable record reconstruction

All interpreter modes shall reconstruct labeled products with `with` under
`TOPAL-TYPE-RECONSTRUCT-001`, preserving the original and every unreplaced
field. Formal traces, source-located diagnostics, LSP validation, and reversible
scripted-debugger history shall cover a commented example.

## TOPAL-INTP-SUBSET-197 — Bound anonymous function values

All interpreter modes shall retain inferred anonymous functions as immutable
values that can be bound and later supplied to compatible List operations under
`TOPAL-FUNCTION-ANONYMOUS-001`. Formal traces, LSP validation, and reversible
scripted-debugger history shall cover commented examples.

## TOPAL-INTP-SUBSET-198 — Direct anonymous function application

All interpreter modes shall directly apply bound inferred anonymous functions
under `TOPAL-FUNCTION-ANONYMOUS-001`, accepting unary operands directly and
multi-parameter operands as positional products. Formal traces, arity
diagnostics, LSP validation, and reversible scripted-debugger history shall
cover commented examples.

## TOPAL-INTP-SUBSET-199 — Short-circuiting traversal control

All interpreter modes shall construct `Continue` and `Finish` control values
and eliminate them in List fold under `TOPAL-EXEC-TRAVERSAL-CONTROL-001`,
without invoking the fold function after `Finish`. Formal traces, LSP
validation, and reversible scripted-debugger history shall cover commented
examples.

## TOPAL-INTP-SUBSET-200 — Symbolic callable values

All interpreter modes shall retain symbolic callables as immutable function
values and apply their unary or positional-product operands under
`TOPAL-FUNCTION-CALLABLE-VALUE-001`. Formal traces, packaging diagnostics, LSP
validation, and reversible scripted-debugger history shall cover commented
examples.

## TOPAL-INTP-SUBSET-201 — Named function values

All interpreter modes shall retain declared functions and their ordered
overloads as immutable values and apply them after rebinding under
`TOPAL-FUNCTION-VALUE-001`. Existing typed selection, recursion, and result
checks shall remain effective. Formal traces, LSP validation, and reversible
scripted-debugger history shall cover commented examples.

## TOPAL-INTP-SUBSET-202 — Lazy iterate generators

All interpreter modes shall construct unbounded `iterate` generators lazily
under `TOPAL-GENERATOR-ITERATE-001`, capturing but not invoking the unary next
function until resumption. Formal traces, LSP validation, and reversible
scripted-debugger history shall cover commented examples.

## TOPAL-INTP-SUBSET-203 — Lazy take-while prefixes

All interpreter modes shall lazily attach `take-while` predicates to generated
traversals under `TOPAL-GENERATOR-TAKE-WHILE-001`, without invoking the next or
predicate function during construction. Formal traces, LSP validation, and
reversible scripted-debugger history shall cover commented examples.

## TOPAL-INTP-SUBSET-204 — Bounded generated foreach

All interpreter modes shall traverse take-while-bounded iterate generators
under `TOPAL-GENERATOR-ITERATE-FOREACH-001`, preserving test-yield-action-resume
order and excluding the first rejected candidate. Formal traces, unbounded
traversal diagnostics, LSP validation, and reversible scripted-debugger history
shall cover commented examples.

## TOPAL-INTP-SUBSET-205 — Finite generated collection

All interpreter modes shall materialize take-while-bounded iterate generators
as Lists under `TOPAL-GENERATOR-COLLECT-001`, preserving classifier, order, and
multiplicity while rejecting unbounded collection. Formal traces, LSP
validation, and reversible scripted-debugger history shall cover commented
examples.

## TOPAL-INTP-SUBSET-206 — Lazy unfold generators

All interpreter modes shall lazily construct seeded unfold generators under
`TOPAL-GENERATOR-UNFOLD-001`, preserving independent seed and yield directions
and deferring the step function until consumption. Formal traces, LSP
validation, and reversible scripted-debugger history shall cover commented
examples.

## TOPAL-INTP-SUBSET-207 — Finite unfold collection

All interpreter modes shall collect unfold generators under
`TOPAL-GENERATOR-UNFOLD-COLLECT-001`, preserving distinct seed and yield
classifiers, yield order, and termination at None. Formal traces, classifier
diagnostics, LSP validation, and reversible scripted-debugger history shall
cover commented examples.

## TOPAL-INTP-SUBSET-208 — Executable root namespace

All interpreter modes shall resolve `root` as the current source-session
namespace and select qualified terminal declarations under
`TOPAL-NAMESPACE-ROOT-001` without flattening or executing the namespace.
Formal traces, diagnostics, LSP validation, and reversible scripted-debugger
history shall cover commented examples.

## TOPAL-INTP-SUBSET-209 — Immutable namespace aliases

All interpreter modes shall bind namespaces as immutable aliases and resolve
qualified members through their retained boundary under
`TOPAL-NAMESPACE-ALIAS-001`. Formal traces, diagnostics, LSP validation, and
reversible scripted-debugger history shall cover commented examples.

## TOPAL-INTP-SUBSET-210 — Namespace availability with use

All interpreter modes shall make namespace paths available with `use` under
`TOPAL-NAMESPACE-USE-001` without flattening members. Formal traces, LSP
validation, and reversible scripted-debugger history shall cover examples.
