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

