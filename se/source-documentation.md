# Source-documentation engineering intent

## Goal

Topal source, built-in declarations, generated reference material, declaration
introspection, and debugger help shall share one documentation meaning so API
descriptions cannot silently diverge between tools.

## Constraints

- `###` documentation remains lossless source trivia and does not affect
  evaluation.
- Attachment is deterministic and structural, including for overloads and
  parameters.
- Documentation generation has no implicit source inputs. Directory recursion
  and built-in `lang` inclusion are explicit user choices.
- Generated reference text is reStructuredText, while source documentation is
  output-format-neutral Unicode prose.
- Built-in and source declarations use the same public documentation view.
- Debugger help resolves the same declaration identities used by the language.

## Validation strategy

Lexer and attachment unit tests establish source behavior. Generator functional
tests cover files, shallow directories, recursive directories, built-in
inclusion, overloads, signatures, and parameter descriptions. Debugger scripts
cover documented identifiers, qualified built-ins, ambiguity, and the command
listing. The documented fundamental library is parsed and generated as a corpus
test.
