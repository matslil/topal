# Best-practice database

Topal maintains a version-controlled best-practice database for three equal
uses: teaching human programmers, guiding agents which develop Topal code, and
driving the Topal linter. A best-practice is established programming guidance,
not a practice exercise. One authoritative record supplies generated human and
agent views and may attach an executable lint rule.

Best-practices do not change language validity. Even a best-practice whose
default lint severity is an error remains configurable and does not silently
become a compiler rule.

## Identity and ownership

Every entry has a stable structured Topal identity. Generic entries belong to
`lang`; external owners use their ordinary namespace:

```text
lang best-practice task state-machine
org.example database best-practice transaction-scope
```

Presentation titles and paths may change without changing identity. A replaced
or renamed entry retains an explicit relationship to its successor. A package
cannot publish entries in a namespace it does not own.

## Status

Status describes the entry's lifecycle:

- `proposed` is under consideration, is never enabled by default, and may
  change before use;
- `active` is accepted for current use;
- `obsolete since V` remains applicable to source selecting an earlier
  language version but is unnecessary or inapplicable from `V`; and
- `deprecated` is no longer considered good guidance independently of language
  version.

Obsolete and deprecated entries remain addressable so old configurations,
suppressions, diagnostics, traces, and links are understandable. Each records
an explanation and may identify a replacement. Obsolescence always records the
language version which made the guidance obsolete.

## Classification and default severity

| Class | Meaning | Normal default |
| --- | --- | --- |
| `template` | a useful structure or ordering to start from and elaborate | warning |
| `recommended` | normally preferable, with documented legitimate exceptions | warning |
| `best-practice` | no known unavoidable exception within its applicability | error |

Template deviations commonly concern style or human comprehension even when
declaration order has no interpreter or compiler meaning. A recommended entry
must document its exceptions. Promotion to `best-practice` requires stronger
review: every apparent exception must have a reasonably clean compliant form.

Every entry separately states whether it is enabled by default and its default
severity. Projects may override severity or disable entries individually, by
owned namespace, or by tag. Exact entry configuration takes precedence over a
broader tag or namespace setting. Scoped suppression uses the same diagnostic
control model as compiler warnings and names the stable best-practice identity.

## Checkability

Strength of guidance and mechanical detectability are independent. An entry
records whether its current lint attachment is `guidance-only`, `heuristic`,
`semantic`, or `formally-decidable`. A heuristic finding explains its
confidence. An automated default error for class `best-practice` requires a
sound semantic or formally decidable check. Universal guidance may remain
guidance-only until such a check exists.

## Applicability and tags

Applicability names the language identity and version range, required or
alternative features, excluded features, required capabilities, source kinds,
and optional platform restrictions. The linter uses each declaration's exact
constructed language context.

Tags are structured identities used for discovery and group policy. Initial
`lang best-practice tag` values include `style`, `readability`, `safety`,
`security`, `performance`, `concurrency`, `architecture`, and
`resource-management`. External databases may add owned tags and should reuse a
`lang` tag when its meaning matches.

## Guidance, rules, and examples

The authoritative entry contains its title, summary, rationale,
recommendation, applicability, exceptions, classification, status, defaults,
checkability, rectification, tags, related identities, specification rules,
provenance, and license. A lint rule is an optional versioned attachment rather
than the definition of the guidance.

Each entry supplies applicable recommended, discouraged, and exception
examples. Example Topal source is shared by the interpreter, debugger, linter,
and future compiler. Comments state what the file demonstrates.

## Lint language variant

Lint-rule source explicitly selects a domain-specific language context:

```topal
use language (
  version is v0.1,
  features is ( lint )
)
```

The variant provides typed read-only operations under `lang lint`. Rules may
inspect lossless tokens, syntax, resolved declarations, typed semantic objects,
package dependencies, or an explicitly supplied execution trace according to
their declared stage. They cannot mutate compiler state or the inspected
program and receive no ambient filesystem, network, process, debugger, or
application authority. Rule execution is deterministic and resource-bounded.

Findings use the shared interpreter/compiler diagnostic structure. Terminal,
JSON, SARIF, and LSP output are adapters over the same finding.

## Rectification

An entry declares rectification as unavailable, a suggestion, or automatic.
An automatic rectification classifies itself as presentation-only,
syntax-preserving, semantics-proven, or review-required. Default `--fix`
applies only the first three, reparses and rechecks changed source, and rejects
overlapping edits. Review-required changes are emitted as patches or
suggestions and are never silently applied.

## External databases and libraries

A source package may include an owned best-practice database and lint rules for
using its libraries. Installing a dependency does not execute or enable its
rules. A project explicitly selects external databases, after which their rules
run in the contained lint context. Integrity, license, supported language
contexts, semantic-view version, dependencies, and requested lint capabilities
are part of package metadata.

## Generated projections

Human reference pages, compact agent records, and compiled lint catalogs are
generated projections and remain committed to version control. Each records
the generator and schema versions, source identity and version, and a digest of
authoritative inputs. An authoritative update changes every affected
projection in the same change. Continuous integration regenerates them and
fails on any difference. Generated files are not edited directly.

## Addition workflow

A new entry begins as `proposed` with an owned identity, classification,
applicability, rationale, defaults, tags, provenance, license, and a positive
example. Activation additionally requires reviewed exceptions, negative and
exception examples where meaningful, current generated projections, lint tests
when a rule exists, rectification tests when a fix exists, and a risk review
appropriate to its classification.

When a language version removes the need for an entry, it becomes `obsolete`
from that version while remaining active for older contexts. When the guidance
itself is rejected, it becomes `deprecated` and stops producing findings by
default. Neither transition deletes historical identity.
