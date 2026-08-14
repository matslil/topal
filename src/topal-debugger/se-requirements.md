# Source debugger requirements

These tool requirements refine `TOPAL-REQ-TOOLS-001`,
`TOPAL-REQ-INTEROP-001`, `TOPAL-REQ-TRACE-001`, and
`TOPAL-REQ-SHARED-001` for the `topal-debug` source execution debugger.

## TOPAL-DEBUG-EXEC-001 — Shared deterministic execution

The debugger shall execute the same accepted language and semantic transitions
as the interpreter through a shared deterministic execution machine. Debugger
observation shall not change program results, semantic decision order, or
diagnostics. Debugger events shall identify stable source and semantic
decisions rather than implementation addresses.

The debugger shall require and honor each debuggee source file's language
selection according to `TOPAL-SYN-CONTEXT-001`. If no version applies to a
tool-created source context, it shall use the highest version supported by the
debugger. Debugger inspection expressions inherit the debuggee's selected
context.

## TOPAL-DEBUG-REVERSE-001 — Reversible Topal state

The debugger shall support reverse stepping by retaining transition history
and reconstructing earlier Topal execution states directly or from deterministic
checkpoints and replay. Re-entering a previously recorded interval shall
reproduce its recorded semantic decisions and values.

The debugger shall not claim to reverse external-world effects. Once effects
or external observations are implemented, replay shall consume their recorded
observations and results without performing them again. A run whose required
record is unavailable shall stop with an explicit diagnostic instead of
silently consulting the current external world.

## TOPAL-DEBUG-CONTROL-001 — Source-level execution control

The command interface shall provide forward and reverse forms of step, next,
finish, and continue where meaningful. It shall support source breakpoints and
expose the current source location, current value, visible bindings, logical
backtrace, and deterministic transition history. Commands that cannot proceed
shall explain why without changing debuggee state.

## TOPAL-DEBUG-MESSAGE-001 — Step into message transactions

Once message passing is supported by the language execution machine, stepping
into a message send shall follow the selected transaction to the receiver's
entry as one source-level control transfer, giving the same continuity as
stepping into a function call. The debugger shall retain and expose the message
transaction identity and sender/receiver relationship, including across task,
process, or machine boundaries represented by the recorded execution.

Reverse stepping across the transfer shall return to the send-side state.
Replay shall follow the recorded receiver selection and response rather than
sending the message again. Step-over shall treat the transaction as one logical
operation while still retaining its nested history for later inspection.

Recorded message transitions shall carry the stable transaction identity and
sender/receiver task identities. Forward semantic stepping from a send shall
follow its matching receive, and reverse stepping from that receive shall
return to the matching send even when unrelated scheduler transitions exist.

## TOPAL-DEBUG-TRACE-001 — Toolchain-comparable history

Debugger transitions shall use a versioned event representation that can be
correlated with interpreter test traces and, later, compiler and runtime
debugger traces. Compatibility is defined by stable semantic decision, source,
and state-transition fields; presentation text and implementation-specific
storage are not comparison fields.

## TOPAL-DEBUG-EXAMPLE-001 — Executable debugger examples

Each implemented debugger capability increment shall add or extend a runnable
Topal source example and an automated debugger scenario exercising that
capability. Language-feature increments shall continue to update LSP coverage
and interpreter examples under their existing requirements.

## TOPAL-DEBUG-MODE-001 — Scripted command mode

`--script COMMANDS FILE` shall debug `FILE` by reading debugger commands from
the named command file, or from standard input when `COMMANDS` is `-`. Script
mode shall not emit prompts and shall produce deterministic command results on
standard output and actionable diagnostics on standard error with a nonzero
status. The debugger functional suite shall exercise debugger behavior through
this mode so the tested interface is also available to users and other tools.

## TOPAL-DEBUG-FAILURE-001 — Inspectable partial failure history

When a diagnostic stops live execution, the debugger shall retain every
completed transition and state checkpoint preceding the failure. The command
session shall remain usable for inspection and reverse navigation rather than
terminating with the debuggee. Repeated advancement may retry only when doing
so cannot duplicate an external effect; otherwise it shall remain stopped at
the recorded failure.

## TOPAL-DEBUG-COMPILER-BOUNDARY-001 — No debugger artifact lowering

The source debugger shall share the interpreter's deterministic GEIR boundary.
Artifact export, compiler lowering, and artifact optimization shall not become
debuggee transitions and shall fail with `E-COMPILER-ONLY` through embedding
interfaces.
