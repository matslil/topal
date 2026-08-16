# Topal linter

`topal-lint` is the batch interface to shared Topal diagnostics and the
best-practice catalog. The first increment validates source through the shared
source/syntax layers and provides deterministic catalog selection:

```text
topal-lint source.t
topal-lint --format json source.t
topal-lint --format sarif source.t
topal-lint --list
topal-lint --explain "lang best-practice task state-machine"
topal-lint --enable "namespace:lang" --disable "tag:lang best-practice tag style" --list
```

An external generated catalog is loaded only when named with `--catalog`.
Stable identity collisions and unsupported schemas are rejected before source
analysis. Installing a library never causes this executable to discover,
enable, or execute its lint code implicitly.

Selectors have three specificity levels: exact identity overrides namespace,
which overrides tag. Later command-line settings win within one level. A
severity of `off` disables the matching entry. Catalog entries without a
reviewed rule attachment never produce a source finding. Rule attachments
embed authenticated Topal source in the generated catalog, so execution does
not grant an external catalog ambient filesystem access. The first rule
receives adjacent task-declaration phases as read-only integer facts and
returns a Boolean decision; the host remains responsible for diagnostics.
Attachments name that input contract explicitly, such as
`task-declaration-order/1` or `task-state-machine/1`, so incompatible view
revisions are rejected before execution.
Admission also caps source bytes and expression nodes and permits only the
bounded pure expression subset supported by the selected view. Unsupported
declarations and potentially unbounded operations fail before rule execution.

Source, syntax, and best-practice findings use `topal-source`'s shared
diagnostic record. Terminal and JSON output are presentation adapters over
that record; best-practice identity and both catalog and rule versions remain
structured provenance rather than text embedded in the message.
SARIF output aggregates every named source into one SARIF 2.1.0 run suitable
for CI ingestion and preserves the same provenance in result properties.
Entries declaring suggestion rectification emit a structured rectification
object in JSON and SARIF and a separate `suggestion` line in terminal output;
the advice is not treated as an automatic edit.

The `topal-linter` library exposes the same engine through `lint_text` for
in-memory consumers such as the language server. It returns shared diagnostic
records and never formats output or reads a source path; the binary remains a
thin filesystem, configuration, and presentation adapter.
