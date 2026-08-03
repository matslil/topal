@/home/mats/.codex/RTK.md

# Topal repository instructions

These instructions apply to every agent working in this repository. Role
descriptions in [`agents/`](agents/) add task-specific responsibilities but do
not replace these rules.

## Authority and conformance

Repository information has the following precedence, from highest to lowest:

1. human-readable language design in `docs/`;
2. system-engineering information in `se/`;
3. tool-specific `se-requirements.md` files;
4. formal language specifications in `spec/`;
5. tests; and
6. implementation in `src/`.

`docs/` is authoritative for design intent. `se/` is authoritative for system
goals and constraints. `spec/` is normative for exact observable language
semantics. The precedence list resolves disagreement by identifying which lower
artifact must change; it does not make deliberately explanatory text a formal
conformance rule.

Verify every substantive change against all applicable information above it,
not only the immediately preceding level. Then propagate approved changes to
all affected lower levels. Maintain links from goals and requirements through
specification rules, tests, and implementation wherever those artifacts exist.

If a requested lower-level change conflicts with a higher level, explain the
conflict and the higher-level change that would be required. Do not proceed
until the human approves that change in the current chat.

## Human decisions

Prior discussion and approval in the current chat are required for substantive
changes to:

- language design in `docs/`;
- system intent, goals, requirements, or strategy in `se/`;
- any tool-specific `se-requirements.md`; or
- fundamental repository policy or design documents at the repository root.

Approval depends on meaning, not location. It is not required for mechanical
moves, spelling, grammar, formatting, repaired links, or other maintenance that
clearly preserves meaning. A simple root-document change does not require prior
approval merely because it is at the root.

Agents may autonomously update `spec/`, tests, and `src/` when the updates
conform to approved higher-level information. If formalization reveals a hole,
contradiction, or necessary design choice, stop and request a human decision
instead of silently choosing new design intent.

## Change procedure

For every change:

1. identify its purpose, affected artifacts, and applicable stable IDs;
2. classify its risk and decide which roles and reviews are warranted;
3. check it recursively against higher-level intent and constraints;
4. obtain chat approval if protected meaning must change;
5. update all affected lower-level artifacts in authority order;
6. validate at each level and update traceability;
7. review the complete diff for coherence and unintended semantic changes; and
8. commit, push, and open a PR for human review.

Downstream updates are automatic in the sense that the agent performs them
without separate prompting after intent is approved. This does not authorize
silent changes to higher-level meaning.

## Risk and review

Participation is based on the risk of error, not diff size. A single agent may
perform and self-review a low-risk change, including acting in several roles.
Use additional roles or independent review when it materially reduces risk.

Consider semantic breadth, subtle interactions, safety, concurrency,
compatibility, formal reasoning, and validation difficulty. Treat changes to
type soundness, memory or concurrency guarantees, serialization compatibility,
security properties, and fundamental language semantics as high risk unless a
concrete assessment justifies otherwise. Record the assessment and review
approach in the PR.

## Pull requests

Every change ends in a PR and requires a human merge decision. Package PRs by
logical cohesion and reviewability, not by authority level. A coherent change
may cross several levels; split work when parts can be understood, validated,
accepted, or reverted independently.

Use the normal permission-escalation process for sandbox restrictions. If a
genuine network or service failure prevents publication, retain the completed
work on a dedicated local branch and report its branch, commits, validation,
and the publication failure.

## Path-specific expectations

- `docs/`: explain the language clearly; expose unresolved ambiguity rather
  than inventing precision inconsistent with approved design.
- `se/`: focus on distinguishing goals and constraints, not a duplicate of the
  full language specification.
- `spec/`: use stable rule IDs, normative language, appropriate formal
  notation, explicit edge cases, and conformance obligations.
- `src/`: keep each tool's `se-requirements.md` beside that tool and connect
  functional tests to specification rule IDs.
- tests: distinguish unit, conformance, interoperability, and specialized
  suites as described by `se/test-strategy.md`.
