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

## Per-test resource regression baselines

`scripts/test_resource_usage.py` runs each libtest case as an exact,
single-threaded invocation in its own systemd user cgroup. A cgroup-local
supervisor obtains cumulative child CPU time and maximum child resident memory
from Linux resource accounting, while systemd enforces the memory and swap
limits. Independent cgroups preserve per-case accounting even when the runner
executes several cases concurrently. The default worker count is bounded by
both the logical CPU count and available memory relative to the per-test memory
limit. Wall-clock duration is deliberately not a conformance metric.

Each measured case runs 50 times in one cgroup by default. The recorded CPU
time is the per-invocation average and peak memory is the maximum across those
samples, reducing noise from process startup and page-level accounting without
weakening the comparison threshold.

Baseline mode records average child CPU time and peak child resident memory for
every discovered test case in `se/test-resource-baseline.json`. Updating that
versioned baseline requires the explicit `--approve-baseline-update` argument
and prior human approval of any unexplained increase. Compare mode fails when a
test is added or removed without a baseline update, a test does not pass, or
either resource metric exceeds its recorded value by more than 20 percent. Such
a deviation requires investigation and a recorded motivation before a human
approves a new baseline; decreases do not rewrite the baseline automatically.

The baseline is meaningful only on a sufficiently comparable execution
environment. Its environment metadata records the architecture, logical CPU
count, operating system, and Rust toolchain. Projects using heterogeneous CI
workers shall establish separate reviewed baselines rather than treating CPU
time from unlike systems as directly comparable.

Create an approved baseline with:

```console
scripts/test_resource_usage.py baseline --approve-baseline-update
```

Compare the current tests with it using:

```console
scripts/test_resource_usage.py compare
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
