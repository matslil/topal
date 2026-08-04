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
under `examples/`. Related features should share a coherent example where that
improves learning; unrelated demonstrations should remain separate. Automated
tests shall execute applicable examples with the interpreter and open them
through the language server, requiring no shared-frontend diagnostics and
verifying relevant editor features. Examples supplement focused conformance
tests and do not replace invalid, boundary, trace, or recovery cases.
