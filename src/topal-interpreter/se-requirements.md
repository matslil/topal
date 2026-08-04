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
shall remain explicitly unsupported until their classification is specified.

## TOPAL-INTP-SUBSET-025 — Record field selection

All interpreter modes shall evaluate `record label` as total static field
selection according to `TOPAL-TYPE-PRODUCT-001`. Selection shall group with its
record before later ordinary application, return the field's exact value, and
diagnose an absent label at the label source range. Test mode shall record the
selected label without evaluating it as a name.
