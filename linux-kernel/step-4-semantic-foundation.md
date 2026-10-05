# Step 4 semantic foundation: initial systems root

## Outcome

This is the first implementation increment of Step 4. It makes the approved
systems profile concrete enough for checked Topal kernel source without
claiming an executable kernel. The initial x86-64 source root is
[the toolchain-gate kernel](kernel/arch/x86_64/toolchain-gate.t).

The shared language frontend now has a separate systems analysis boundary
which:

- accepts only the explicit `x86_64-unknown-none`,
  `topal-qemu-pc-q35-10.2`, and
  `topal.systems.x86_64-qemu-pc-q35-10.2/1` combination;
- checks one bootstrap entry and one debug-break synchronous-exception entry;
- retains console-write, debug-break, resume, and fatal operations by stable
  semantic identity;
- derives the exact handler effect set;
- prevents entry contexts from escaping or being used outside admitted
  operations;
- requires one context-consuming final disposition; and
- exposes a deterministic abstract transition trace.

The ordinary Linux-process compiler continues to reject the `systems` feature.
The new kernel target appears in `topalc --list-targets` as model-only and
fails with its missing qualification scopes before frontend lowering or output
publication.

## Abstract evidence

The checked toolchain-gate root models this transition sequence:

1. enter the bootstrap handler;
2. write `TOPAL_KERNEL_BOOT` through the borrowed board console;
3. observe a declared debug break;
4. enter the typed debug-break handler;
5. consume its context with the resume disposition;
6. write `TOPAL_KERNEL_FAULT_RESUMED`; and
7. consume the bootstrap context with the fatal disposition.

Negative tests cover the hosted target, wrong board, context escape, unknown
machine operation, missing disposition, wrong handler signature, open entry
set, invalid abstract-model construction, and artifact-free model-only target
failure.

## Validation

- all 46 `topal-semantics` library tests pass;
- all 178 `topal-compiler` library tests and all 259 compiler CLI tests pass;
- all five focused systems source-checker tests pass;
- the four core-language coverage and traceability tests pass; and
- strict Clippy passes for `topal-semantics` and `topal-language`. The changed
  compiler path passes when the compiler crate's four pre-existing lint
  findings are allowed.

The full `topal-language` library run has one existing failure in
`models_direct_bounded_int_iterate_collection`: the unchanged test expects the
current analyzer to reject an indirect bounded collection, but that analyzer
returns a checked program. This increment does not alter that compiler-model
path or its test.

## Boundary and next increment

No LLVM lowering, privileged instruction, object, boot image, or QEMU result is
introduced here. That is deliberate: `TOPAL-SYSTEMS-QUALIFY-001` requires the
semantic boundary to fail closed until a provider has physical evidence.

The next increment will add bounded bootstrap storage to the checked model,
then implement the x86-64 entry, 16550 console, debug-break/resume, fatal, and
artifact-publication provider. Only after structural inspection and a boot in
the pinned Step 3 VM may the target move from model-only to
executable-qualified.

## Risk and review

Risk is high because this establishes privileged control-flow and authority
semantics. The implementation remains isolated from ordinary compilation,
admits a deliberately closed vocabulary, assigns stable semantic identities,
models every accepted transition, and rejects physical publication. The
change was self-reviewed against `TOPAL-SYSTEMS-FEATURE-001`,
`TOPAL-SYSTEMS-VOCABULARY-001`, `TOPAL-SYSTEMS-AUTHORITY-001`,
`TOPAL-SYSTEMS-ENTRY-001`, `TOPAL-SYSTEMS-DISPOSITION-001`,
`TOPAL-SYSTEMS-OBSERVATION-001`, and `TOPAL-SYSTEMS-QUALIFY-001`.
