# Toolchain architecture

This architecture realizes `TOPAL-GOAL-TOOLCHAIN-001` and
`TOPAL-REQ-SHARED-001`. It records approved system intent for the interpreter,
compiler, language server, linter, and static and runtime debuggers.

## Shared pipeline

Tools consume progressively richer reusable layers:

```text
source text and line map
        |
        v
lossless tokens and recoverable syntax
        |
        v
resolved semantic model and typed representation
        |
        +--> interpreter and runtime debugger
        +--> compiler and compiled debug metadata
        +--> language server and completion
        `--> built-in and custom lint rules
```

The source layer owns decoding, normalization, byte ranges, and line/column
mapping. The syntax layer retains tokens and trivia even for malformed or
incomplete input; it reports diagnostics without requiring evaluation. The
semantic layer owns names, types, capabilities, effects, and stable
relationships. Execution engines consume semantic output rather than embedding
their own source grammar.

## Stability boundaries

Source ranges use normalized UTF-8 byte offsets and remain convertible to
line/column positions. Syntax nodes and semantic objects may evolve internally,
but tool-facing identities shall be deterministic for one source revision.
Diagnostics carry stable codes and source ranges. Test traces and debugger
events refer to semantic decisions and source identities rather than memory
addresses or evaluator implementation details.

## Reversible source execution

The interpreter and source debugger share a deterministic execution machine.
The machine exposes source-level transitions whose stable identities, semantic
decisions, and state changes can be consumed without coupling tools to its
internal call stack. The interpreter runs those transitions to completion; the
debugger retains enough transition history and periodic checkpoints to inspect
earlier Topal states and to replay forward deterministically.

Reverse execution reverses or reconstructs Topal execution state. It does not
attempt to undo changes already made to the external world. When later language
features observe external state or perform effects, a recorded debugging run
shall replay the recorded observations and results rather than repeating the
effect. Compiler and runtime-debugger event streams shall be comparable at this
shared transition boundary.

Source-level stepping follows semantic control transfer rather than merely
moving between physical stack frames. In particular, once message passing is
implemented, stepping into a send follows the selected message transaction to
its receiver entry with the same user-facing continuity as stepping into a
function call. The transaction identity remains visible so that asynchronous
or distributed transfer is not misrepresented as an ordinary local call.

Custom lint APIs will consume an explicitly versioned read-only semantic view;
they shall not mutate compiler or interpreter state. Editor recovery may retain
missing or erroneous syntax, but batch acceptance still follows the closed
formal grammar.
