# Data-transfer Topal test migration plan

This plan moves portable standard-library expectations out of Rust assertions
and into executable Topal source. It covers the currently published design-0
slice in `library/std/{data,transfer,store,network,device}` and the firewall
example. It does not claim completion of the wider data-transfer architecture;
those gaps remain in the conformance matrix.

## Authority and stable IDs

The tests implement the behavior approved in `docs/data-transfers.md` and the
contracts in `spec/data-transfer-packages.md` and `spec/data-transfers.md`.
Applicable requirements are `TOPAL-TRANSFER-PACKAGE-001` through `005`, the
phase-specific `TOPAL-DATA-*`, `TOPAL-TRANSFER-*`, `TOPAL-STORE-*`,
`TOPAL-NETWORK-*`, and `TOPAL-DEVICE-*` IDs, and `TOPAL-REQ-TOOLS-001` for
shared source-tool handling.

## Test ownership

Portable behavior and expected values belong in `.t` files beneath
`tests/standard-library`. Each file defines a `Pass` constraint and constructs
classified evidence for every expectation. A false expectation therefore
fails during Topal evaluation; the host harness does not contain a duplicate
golden value.

Rust may retain only infrastructure and boundary tests:

- discover and evaluate every Topal test source against the real library tree;
- verify that static tools accept the same corpus;
- test parser, evaluator, test discovery, diagnostics, and tracing themselves;
- test irreducible OS calls, handles, syscalls, controller access, and native
  error translation; and
- characterize the legacy Rust reference backend while it remains in-tree.

Tests of that legacy backend are not standard-library conformance evidence.

The compact `path-coverage` tables described in `docs/testing.md` are not yet
implemented by the v0.1 execution subset. Constraint-based Topal tests are the
executable migration mechanism now. Once compact tables are implemented, they
may replace the constraint boilerplate without changing test ownership.

## Native GitHub stack

1. **Plan and audit.** Record ownership, migration rules, coverage inventory,
   and the disposition of completed plans.
2. **Topal test runner.** Add one expectation-free discovery harness, remove
   Rust assertions for the new source API and firewall behavior, and prove that
   a deliberately failing Topal expectation fails the harness.
3. **Data and transfer laws.** Cover span bounds, overlap, scatter/gather,
   queue bounds and ordering, completion representation, and retry evidence.
4. **Store, network, device, and firewall laws.** Cover identity lookup,
   guarantee ranks, address families and boundaries, I2C boundaries and
   limits, and source-level firewall decisions.
5. **Cross-tool and conformance audit.** Feed the corpus to the linter and LSP,
   distinguish source evidence from native/reference evidence, update
   traceability, and remove completed implementation plans that no longer
   govern unfinished work.

## Coverage obligations

Every public operation in the current slice receives normal, boundary, empty,
and rejection cases where its domain permits them. Laws include symmetry,
boundary-touch non-overlap, FIFO preservation, family separation, stable store
identity, and composition of independent firewall views. Test names state the
law or boundary rather than an implementation detail.

The plan deliberately does not manufacture tests for APIs that do not yet
exist. Cancellation races, owned regions, validated-view invalidation,
stream/message adapters, file and database stores, native network transports,
DMA, and full virtual I2C remain deferred in
`data-transfer-conformance-matrix.md` and require implementation before their
Topal conformance suites can close.

## Acceptance

Each layer passes `git diff --check`, focused tests, and the complete workspace
suite under a 4 GiB `systemd-run` memory limit with swap disabled. The existing
recursive evaluator tests require an 8 MiB Rust test-thread stack. The final
stack must be clean, correctly based, published as draft PRs by `gh stack`, and
reviewed as a coherent cumulative diff.
