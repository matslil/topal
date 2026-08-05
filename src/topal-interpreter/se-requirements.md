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
