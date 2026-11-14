# Step 4: reviewable systems implementation

`TK-DEC-277`; implements the [maintenance plan](design/modular-systems-implementation.md).
The preceding [instance qualification](step-4-instance-scoped-region-actions.md)
identified six already oversized Rust files and ignored cached dependencies in
the source-size audit. This increment restores that audit without changing
language meaning, normative systems contracts or kernel capabilities. The
[status ledger](step-4-status.md) retains the unfinished initial-plan obligations.

## Source organization and preserved behavior

Complete declarations, functions and test groups move to included files in the
same lexical Rust modules. Public type paths, private access, imports, macro
contexts, relative fixture paths and test identities are preserved. Large
inherent implementations reopen the same type rather than changing traits or
exposing a new interface. Every owned Rust source retains the 2000-line ceiling;
the largest newly extracted source has 1949 lines.

| Original owner | Preceding lines | Current facade lines |
| --- | ---: | ---: |
| `topal-language::systems` | 15133 | 1809 |
| `topal-language::systems_translation_edit` | 6104 | 282 |
| `topal-semantics::systems` | 12823 | 1810 |
| `topal-compiler::systems_artifact` | 14972 | 1163 |
| `topal-compiler::systems_provider` | 2087 | 1558 |
| `topal-compiler::systems_provider_object` | 24863 | 469 |

The context-decision checker delegates its remaining user lifecycle with the
same completion identity and ordered worker-operation prefix. The static-image
checker shares 83 identical marker checks with the original analysis, comparison,
diagnostic text and source-span order. The native process emitter delegates five
ordered phases through its one encoder and unchanged flags. The instruction
audit retains its single test entry and every assertion through eight ordinary
helpers with explicit shared observations.

The [preservation scripts](tools/systems-source-audit/README.md) compare the
qualified predecessor with the extracted modules. They match 908 unchanged
function token/literal fingerprints and 411 declarations/imports/macros/module
gates. No function or declaration is removed. Four reviewed parent routines
delegate to fifteen new private helpers. The backend comparison additionally
verifies the exact ordered instruction/control/label/relocation emission tape,
copied constant values, phase arguments and complete original assertion tape.
Functional tests remain necessary for the two checker decompositions.

The size audit prunes `.git`, `target` and the generated `linux-kernel/.cache`
dependency tree before descent. Other `.cache` directories remain checked. It checks all 381 owned Rust sources, including every tracked source
and new untracked source. It adds no source exemption or higher ceiling. A
filesystem control confirms that new and nested owned files remain visible.

## Native identity and provenance

The primary kernel SHA-256 remains
`397e07bfa0c0528e73389ea781e112923b2637a5aa1ee587e4a795cc2cbc7bb0`.
Source libraries, native code/data, caller identities, ABIs, resource certificates,
boot image/resource and semantic trace retain their preceding contracts.
Artifact/root/provider/object/boot/gate revisions remain 250/236/237/229/150/250,
with 297 unchanged normative rules. The private frame family remains
`c33b5beb0f1869341bf24493e867f5d3101a578a12962b525c2613c1e09e3bbe`.

An independent checkout of the qualified predecessor builds the same artifacts
against the unchanged Topal sources. Existing tool text carries temporary paths:
four retained LLVM ModuleID headers and five object-disassembly headers. The
predecessor's own two builds demonstrate different diagnostic hashes on five
byte-identical policy objects. This is an existing property of provenance,
without a production format or hash-calculation change in this increment.

The [provenance comparison](labs/qemu/x86_64/modular_provenance.py) verifies every
published byte and its recorded checksum. Only the four exact ModuleID headers
may differ in published text; their complete module bodies must match. It
reconstructs and verifies each original disassembly checksum using the identical
object and recorded temporary work path before normalizing that one header.
Every remaining provenance field must match. Controlled changes to an object,
LLVM body, resource fact and disassembly checksum must each be refused.

## Qualification and review

The complete bounded Rust workspace passes, including all 621 language, 173
semantics, 254 compiler unit and 261 compiler CLI tests, eight action integrations,
two Unit-data integrations and both native frame/instance GDB integrations.
Workspace all-target checks, workspace formatting and 80 explicitly included
source formatting checks pass. All 41 Python lab contracts, both preservation
audits and the complete owned-source size audit pass.

The [fresh primary VM record](labs/qemu/x86_64/results/topal-modular-systems-gate.json)
passes the independent artifact/provenance comparison and all four controlled
refusals. It binds the harness, comparison and preservation tools by checksum.
The common normalized provenance SHA-256 is
`f172f059f7ff184c8886bf666efa14def0f2158dce2a2a3ea8808f6df183ad65`.
Two exploratory boots first passed their native execution oracles but stopped
on the original whole-record checksum assertion. Investigating that failure
produced the verified header comparison above; no native discrepancy was waived.

The preceding 137 VM observations, including forty BusyBox workloads and explicit
legacy root/action/frame controls, remain separately identified retained evidence.
They must not be described as a fresh full-cohort run in this increment.

Risk is high at namespace/macro boundaries, early exits, checker spans and
operation order, shared encoder state, literal templates, labels/relocations and
lost assertions. Self-review combines the higher-level preserved contracts,
token/literal inventories, exact emitted artifact comparison, existing functional
tests and controlled refusals. Human review remains required; no independent
reviewer is claimed. Execution uses isolated QEMU TCG and bounded host validation.

The four stages have separate commits: plan, checker/model extraction,
artifact/provider/audit extraction, and final qualification. They update the
existing Step 4 PR. Distribution replacement, Docker/Podman, VM hosting, Device
Tree consumption and the other ledger items remain initial-plan work.
