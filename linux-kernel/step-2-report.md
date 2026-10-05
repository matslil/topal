# Step 2 — Topal-native kernel design proposal

## Outcome

The project now has a coherent internal kernel architecture, a many-to-many
mapping from Linux 7.2.9 responsibilities into that architecture, and a
minimal proposed Topal systems profile for operations that cannot be expressed
by the current portable language.

The design concludes that a Linux-compatible kernel is feasible in principle,
but current Topal is missing mandatory semantics and backend support. It does
not recommend porting Linux C structure-by-structure, adding inline assembly,
or introducing a generic unsafe mode.

## Design set

| Artifact | Purpose |
| --- | --- |
| [kernel architecture](design/topal-kernel-architecture.md) | layer, boot, execution, memory, scheduler, object, UAPI, security, driver, failure, and evidence design |
| [gap analysis](design/topal-gap-analysis.md) | separates reusable Topal foundations from mandatory language/toolchain additions |
| [systems-profile proposal](design/kernel-elements.md) | defines the semantic contract and library boundary for each new element family |
| [architecture pressure test](design/architecture-pressure-test.md) | checks the abstraction against x86-64, AArch64, and RISC-V without promising non-x86 implementation |
| [Linux mapping ledger](design/linux-to-topal-mapping.md) | maps responsibilities and defines future-baseline drift procedure |
| [decision register](design/decision-register.md) | records which recommendations still need protected-design approval or VM qualification |

## Principal choices proposed for approval

1. select a separate capability-gated, freestanding Topal systems profile;
2. preserve existing portable Topal meaning rather than introduce `unsafe`;
3. use backend-generated typed special entry and return dispositions instead
   of source assembly;
4. admit external events and concurrent winner selection only through declared,
   traceable protocols rather than general nondeterminism;
5. expose sealed semantic machine-provider operations, keeping genuinely
   target-specific facilities target-qualified;
6. add typed address-space/mapping and recoverable user-transfer elements;
7. add atomic locations and scoped critical state integrated with Topal effects,
   resources, contexts, and race rejection;
8. express CPU, device, DMA, translation, cache, and instruction ordering as
   distinct semantic relations;
9. make scheduler context transfer and fatal disposition explicit nonordinary
   control transitions;
10. publish a separate kernel artifact with no Linux-process runtime; and
11. keep Linux policy and subsystems in libraries/modules rather than language
    syntax or compiler intrinsics.

## Approval record

The proposal was approved in the project discussion after PR #790 merged. The
approved meaning is propagated in a follow-up authority-ordered change through
`docs/systems-profile.md`, `TOPAL-REQ-SYSTEMS-*`,
`spec/systems-profile.md`, tool requirements, and traceability. Implementation
and executable x86-64 qualification remain planned rather than implied by
design approval.

## Risk and review assessment

The proposed semantics affect privilege, memory safety, concurrency, and
fundamental control flow, so adoption is high risk even though this PR contains
documentation only. The review controls are:

- explicit separation of proposals from authority;
- cross-architecture pressure testing;
- rejection of general escape hatches;
- one stable identity per responsibility and element;
- documented negative/context/lifetime/artifact evidence for qualification;
- preservation of Linux external behavior at adapters; and
- a required human approval checkpoint before authoritative changes.

## Next work after design adoption

1. implement architecture-independent semantic checking and model behavior;
2. implement and qualify the x86-64 providers and kernel artifact toolchain;
3. use the independently reproducible Step 3 Linux VM as the boot and
   differential reference; and
4. begin kernel implementation only after the necessary toolchain gate passes.
