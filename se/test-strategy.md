# Test strategy

Topal uses the following primary test classes:

- **Unit tests** verify that isolated algorithms and data structures behave as
  intended.
- **Functional conformance tests** verify observable tool behavior against
  cited formal specification rules, including compile-pass, compile-fail,
  diagnostics, and runtime behavior.
- **Interoperability tests** run equivalent inputs through the interpreter and
  compiled output and compare their semantic observations.

Specialized suites refine those classes where needed:

- parser ambiguity, recovery, and diagnostic tests;
- serialization golden vectors, malformed inputs, and version compatibility;
- memory and concurrency litmus tests for allowed and forbidden executions;
- property-based, metamorphic, and fuzz tests;
- end-to-end toolchain tests; and
- performance regressions tied to explicit performance requirements.

## Separate implementation and Topal test systems

Rust implementation tests and Topal source tests have separate owners and
entry points. `cargo test` verifies the Rust implementations of the language
tools, shared semantic machinery, native boundaries, and the small bootstrap
surface needed to launch the Topal test runner. It shall not discover or own
the Topal standard-library or application test corpus.

`topal test` discovers `.t` programs recursively and verifies Topal libraries,
applications, examples, and observable conformance behavior. Every file has an
independent stable path identity and may be listed, selected exactly, filtered,
or run in bounded parallelism. A constraint rejection, diagnostic, unreadable
test, or failed application expectation fails that Topal test and the command.

Run the implementation tests with:

```console
cargo test --workspace --all-targets
```

Run the standard-library Topal corpus with:

```console
topal test tests/standard-library examples/data-transfer
```

This bootstrap separation does not imply that the Rust implementation is part
of the Topal language design. An independently implemented Topal toolchain may
provide the same `topal test` behavior without Rust.

## Per-test resource regression baselines

`scripts/test_resource_usage.py --domain rust` runs each libtest case as an exact,
single-threaded invocation in its own systemd user cgroup. A cgroup-local
supervisor obtains cumulative child CPU time and maximum child resident memory
from Linux resource accounting, while systemd enforces the memory and swap
limits. Independent cgroups preserve per-case accounting even when the runner
executes several cases concurrently. The default worker count is bounded by
both the logical CPU count and available memory relative to the per-test memory
limit. Wall-clock duration is deliberately not a conformance metric.

`scripts/test_resource_usage.py --domain topal` separately discovers every
identity reported by `topal test --list` and measures an exact, single-job
invocation of that Topal test in its own cgroup. Rust and Topal use distinct
versioned baselines, `se/test-resource-baseline.json` and
`se/topal-test-resource-baseline.json`, so removing a Rust wrapper cannot hide
or merge the resource behavior of the Topal files it formerly aggregated.

Each measured case runs 50 times in one cgroup by default. The recorded CPU
time is the per-invocation average and peak memory is the maximum across those
samples, reducing noise from process startup and page-level accounting without
weakening the comparison threshold.

Baseline mode records average child CPU time and peak child resident memory for
every discovered test case in the selected domain's baseline. Extending that
versioned baseline requires the explicit `--approve-baseline-update` argument.
When the file already exists, ordinary baseline mode adds only newly discovered
test identities: existing measurements and metadata remain unchanged, including
entries absent from the current run. Replacing existing measurements requires
the additional `--replace-existing-baseline` argument and prior human approval
of every motivated change. Compare mode fails when a test is added or removed
without a baseline update, a test does not pass, or either resource metric
exceeds its recorded value by more than 20 percent. Such a deviation requires
investigation and a recorded motivation before a human approves a replacement;
decreases do not rewrite the baseline automatically.

The baseline is meaningful only on a sufficiently comparable execution
environment. Its environment metadata records the architecture, logical CPU
count, operating system, and Rust toolchain. Projects using heterogeneous CI
workers shall establish separate reviewed baselines rather than treating CPU
time from unlike systems as directly comparable.

Create an approved baseline with:

```console
scripts/test_resource_usage.py baseline --domain rust --approve-baseline-update
scripts/test_resource_usage.py baseline --domain topal --approve-baseline-update
```

Replace existing measurements only after explicit human approval:

```console
scripts/test_resource_usage.py baseline --domain rust --approve-baseline-update --replace-existing-baseline
scripts/test_resource_usage.py baseline --domain topal --approve-baseline-update --replace-existing-baseline
```

Compare the current tests with it using:

```console
scripts/test_resource_usage.py compare --domain rust
scripts/test_resource_usage.py compare --domain topal
```

Functional and interoperability tests shall cite stable specification IDs.
Tests shall cover valid behavior, invalid behavior, boundaries, and interactions
between specification domains. Coverage quantity alone does not establish
semantic completeness.

## Language-feature increment completeness

Every increment which adds or extends observable language behavior shall update
all affected shared frontend and semantic layers, interpreter behavior, and
formal traces in the same reviewable series. It shall also update the language
server implementation or its explicit conformance coverage so editor behavior
recognizes, diagnoses, and presents the feature consistently with batch tools.

Each such increment shall add or extend at least one runnable Topal source file
under `examples/language/`. These source examples are tool-neutral fixtures:
the interpreter, source debugger, language server, and future compiler shall
consume the same file whenever they exercise that language feature. Related
features should share a coherent example where that improves learning;
unrelated demonstrations should remain separate. Tool-specific directories may
contain source only when the source itself demonstrates tool control, history,
failure, or another tool-specific concern. Debugger command scripts and similar
non-source inputs remain tool-specific.

Shared malformed or failing source belongs under
`examples/language-diagnostics/` when several tools exercise the same language
diagnostic. These files are excluded from the successful runnable corpus but
shall be consumed directly by the applicable interpreter, debugger, language
server, and future compiler tests.

Automated tests shall execute applicable shared examples with the interpreter,
open them through the language server, and use them in debugger or compiler
scenarios where relevant, requiring no shared-frontend diagnostics and
verifying relevant tool behavior. Examples supplement focused conformance tests
and do not replace invalid, boundary, trace, or recovery cases.
