# Validation strategy

Validation asks whether Topal specifies the intended language. For each goal
and requirement, maintainers shall review representative user scenarios and
confirm that the human-readable design expresses the desired outcome.

Validation evidence includes:

- scenario reviews for application, library, tool, and systems stakeholders;
- design examples demonstrating composability, safety, and diagnostics;
- explicit review of deferred or rejected behavior;
- cross-tool interoperability demonstrations once implementations exist; and
- human approval of substantive changes to design intent or requirements.

Architecture-model validation additionally uses representative scenarios for:

- a generic host-family target which does not acquire optional host features;
- a named specific processor and a foreign cross-compilation target;
- CPU, GPU, NPU, DSP, heterogeneous-memory, DMA, coherent and noncoherent
  platform compositions;
- board overlays with external memory, cache, bus, and device connections;
- conflicting, incomplete, uncertain, and dimensionally invalid facts;
- cache, false-sharing, interconnect-bottleneck, SIMD, layout, tiling, transfer,
  and multiversioning decisions which combine program and hardware facts; and
- preservation of the ordinary-source authority boundary and a correct
  conservative fallback when cost evidence is missing.

Measured validation records the exact model, board, firmware, operating mode,
toolchain, method, workload shape, and tolerance. It validates a qualified cost
claim, not portable language semantics or a universal performance guarantee.

Optimization-policy validation additionally presents the same candidate plans
under `O0`, speed-first, size-first, memory-first, and explicit-limit scenarios.
Reviews confirm that infeasible plans never win, dominated plans are removed,
unknown facts supply no preference, incomparable plans follow the documented
ordered goals, and ties are independent of enumeration order. Generic-host,
specific-host, native, board, and cross-target scenarios confirm that target
selection remains independent from optimization level. Pass isolation, later
enable/disable precedence, listing, explanation, and missing-information
diagnostics are reviewed without treating mandatory lowering as optional.

Validation is performed whenever a design or requirement changes and before a
language revision is declared stable. Open ambiguity that could alter
observable behavior is a validation finding, not an implementation choice.
