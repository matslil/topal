# Systems implementation preservation audit

TK277 compares the extracted implementation with qualified commit
`73e17ab5a50d34561b71db6fb835ed0aae31217f`. Run from the repository root:

```sh
rtk scripts/run_bounded.py -- python linux-kernel/tools/systems-source-audit/checkers.py
rtk scripts/run_bounded.py -- python linux-kernel/tools/systems-source-audit/backends.py
```

The scripts recursively expand included items in their original lexical module,
retain literal contents, and compare function/declaration fingerprints. They
normalize Rust formatting in an equal context. The backend audit additionally
flattens the five process emitter phases and eight instruction audit phases,
checking their exact ordered statements and literal values. It checks copied
local emitter constants against the predecessor. This is deliberately tied to
the maintenance checkpoint, rather than a substitute for compiler conformance.

Four reviewed parent functions delegate to fifteen new private helpers. The
checker decomposition also needs its source/span/operation tests; token
comparison alone does not prove the two delegated checker algorithms. These
scripts complement the complete Rust suite, actual generated artifact identity,
VM execution and the unchanged normative systems contracts. Their small lexical
scanner is not a general Rust parser; unsupported forms fail their assertions.
