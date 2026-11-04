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

For an independent native artifact comparison, create a detached checkout of
that same predecessor. With the same bounded environment used for workspace
validation (one Cargo job, no debug information or incremental compilation,
1024 codegen units and a 16 MiB Rust stack), run:

```sh
rtk git worktree add --detach /tmp/topal-tk277-baseline 73e17ab5a50d34561b71db6fb835ed0aae31217f
rtk scripts/run_bounded.py -- python linux-kernel/tools/systems-source-audit/build_baseline.py --checkout /tmp/topal-tk277-baseline --output /tmp/topal-tk277-baseline-artifact
rtk scripts/run_bounded.py -- python linux-kernel/labs/qemu/x86_64/modular_gate.py --baseline-artifact /tmp/topal-tk277-baseline-artifact/linked
```

Choose unused output paths. The builder verifies the pinned checkout and the
unchanged Topal kernel sources. The VM harness compares every published byte
and all provenance fields with the independent artifact. Existing `llvm-dis`
ModuleID headers and `llvm-objdump` object-path headers contain temporary build
paths. It verifies their original recorded hashes before normalizing just those
headers, preserving every instruction, module body and resource fact. Four
controlled changes to object bytes, LLVM body, resource metadata and disassembly
checksum must be refused. The original 137-observation cohort remains separately
identified as retained evidence; the new record reports one fresh primary VM.
