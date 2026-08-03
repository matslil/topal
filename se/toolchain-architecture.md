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
semantic layer will own names, types, capabilities, effects, and stable
relationships. Execution engines consume semantic output rather than embedding
their own source grammar.

## Stability boundaries

Source ranges use normalized UTF-8 byte offsets and remain convertible to
line/column positions. Syntax nodes and semantic objects may evolve internally,
but tool-facing identities shall be deterministic for one source revision.
Diagnostics carry stable codes and source ranges. Test traces and debugger
events refer to semantic decisions and source identities rather than memory
addresses or evaluator implementation details.

Custom lint APIs will consume an explicitly versioned read-only semantic view;
they shall not mutate compiler or interpreter state. Editor recovery may retain
missing or erroneous syntax, but batch acceptance still follows the closed
formal grammar.
